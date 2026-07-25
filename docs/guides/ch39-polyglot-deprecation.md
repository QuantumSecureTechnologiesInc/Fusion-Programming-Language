# Chapter 39: The Polyglot Exit Strategy

Every polyglot system eventually faces a reckoning. Languages evolve, teams change, maintenance burden accumulates, and the elegant polyglot architecture from 2024 becomes the legacy monstrosity of 2027. This chapter covers how to migrate away from polyglot without burning down the system.

## When Polyglot Was a Mistake

Not every polyglot decision was good. Here's how to tell if yours was a mistake:

### Signs You Should Consolidate

```
Red Flag Scorecard:
□ More than 30% of bugs occur at language boundaries
□ Team spends >20% of time on build/CI plumbing
□ Onboarding takes >3 months for any single service
□ You have more build configurations than features
□ "It works on my machine" happens across languages
□ Deploy failures correlate with language version upgrades
□ Two or more languages implement the same business logic

Score ≥4: Consolidation is probably warranted
```

### The Real Cost of Polyglot

```python
# Hidden costs that accumulate over time

costs = {
    "toolchain_maintenance": {
        "rust_upgrades": "40 hours/year",
        "python_version_pinning": "20 hours/year",
        "go_mod_tidy": "10 hours/year",
        "cross_compiler_config": "30 hours/year",
    },
    "knowledge_loss": {
        "expert_leaves_team": "200+ hours to replace",
        "only_one_person_knows_rust_ffi": "Critical bus factor",
        "new_hire_training": "3-6 months per language",
    },
    "operational_burden": {
        "monitoring_per_language": "N separate dashboards",
        "logging_correlation": "Manual trace stitching",
        "debugging_cross_language": "2-5x longer resolution",
    },
}

# Total annual cost of a 3-language polyglot system:
# ~$180,000-$250,000 in engineering time alone
# (assuming $150/hour loaded cost, 1200-1700 hours/year)
```

### The Sunk Cost Fallacy

"We already invested 6 months in the Rust FFI layer, so we can't remove it" is not a valid argument. The question isn't what you spent — it's what maintaining it costs going forward vs. the alternatives.

## API Gateway Gradual Migration

The API Gateway pattern lets you migrate service-by-service while keeping the old system running.

### The Strangler Fig Migration

```
Phase 1: New gateway routes 10% to new service, 90% to old
         ┌─────────────┐
         │   Gateway   │
         │  (10% new)  │
         └──────┬──────┘
          10%   │   90%
          ┌─────┴─────┐
          ▼           ▼
    ┌──────────┐ ┌──────────┐
    │  New     │ │  Old     │
    │  Service │ │  Service │
    │  (Go)    │ │  (Rust)  │
    └──────────┘ └──────────┘

Phase 2: Route 50% to new, 50% to old (canary testing)

Phase 3: Route 100% to new, old service stays for rollback

Phase 4: Decommission old service
```

### API Gateway Configuration

```yaml
# envoy.yaml — traffic splitting during migration
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address: { address: 0.0.0.0, port_value: 8080 }
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                route_config:
                  virtual_hosts:
                    - name: backend
                      routes:
                        - match: { prefix: "/api/v2" }
                          route:
                            weighted_clusters:
                              clusters:
                                - name: new_service
                                  weight: 10
                                - name: old_service
                                  weight: 90
                            retry_policy:
                              retry_on: "5xx"
                              num_retries: 3

  clusters:
    - name: new_service
      connect_timeout: 1s
      load_assignment:
        cluster_name: new_service
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: new-service
                      port_value: 8081
    - name: old_service
      connect_timeout: 1s
      load_assignment:
        cluster_name: old_service
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: old-service
                      port_value: 8082
```

### Feature Flags for Migration

```python
# feature_flags.py — migration feature flags
from enum import Enum

class MigrationPhase(Enum):
    RUST_ONLY = "rust_only"        # 100% old system
    CANARY = "canary"              # 10% new, 90% old
    GRADUAL = "gradual"            # 50/50
    NEW_PRIMARY = "new_primary"    # 90% new, 10% old
    COMPLETE = "complete"          # 100% new

# Feature flag configuration
MIGRATION_CONFIG = {
    "user_service": MigrationPhase.GRADUAL,
    "order_service": MigrationPhase.CANARY,
    "payment_service": MigrationPhase.RUST_ONLY,  # Don't migrate
    "notification_service": MigrationPhase.COMPLETE,
}

def should_use_new_service(service_name: str, user_id: str) -> bool:
    """Determine whether to route to new or old service."""
    phase = MIGRATION_CONFIG.get(service_name)
    if phase == MigrationPhase.RUST_ONLY:
        return False
    if phase == MigrationPhase.COMPLETE:
        return True
    if phase == MigrationPhase.CANARY:
        # Route 10% based on user_id hash
        return hash(user_id) % 100 < 10
    if phase == MigrationPhase.GRADUAL:
        return hash(user_id) % 100 < 50
    return False
```

## Database Migration Across ORMs

Different languages often use different ORMs. Migrating the database while changing ORMs is one of the riskiest operations in polyglot migration.

### The ORM Layer Problem

```
Before Migration:
  ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ Rust     │────▶│ Diesel   │────▶│ Postgres │
  │ Service  │     │ ORM      │     │          │
  └──────────┘     └──────────┘     └──────────┘
  ┌──────────┐     ┌──────────┐
  │ Python   │────▶│ SQLAlchemy│
  │ Service  │     │ ORM      │
  └──────────┘     └──────────┘

After Migration:
  ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ Go       │────▶│ sqlc     │────▶│ Postgres │
  │ Service  │     │ (generated)│   │          │
  └──────────┘     └──────────┘     └──────────┘
```

### Migration Strategy: Dual-Write

```python
# dual_write.py — Write to both ORMs during migration
from contextlib import contextmanager

class DualWriteUserRepository:
    """Writes to both old (SQLAlchemy) and new (raw SQL) repositories."""

    def __init__(self, old_repo, new_repo):
        self.old_repo = old_repo
        self.new_repo = new_repo

    def create_user(self, user: User) -> User:
        # Write to new system first (it's the future)
        new_user = self.new_repo.create(user)
        # Write to old system (for rollback capability)
        old_user = self.old_repo.create(user)

        # Verify consistency
        if new_user.id != old_user.id:
            raise InconsistentWriteError(
                f"ID mismatch: new={new_user.id}, old={old_user.id}"
            )

        return new_user  # Return from new system

    def get_user(self, user_id: str) -> User:
        # Read from new system
        try:
            return self.new_repo.get(user_id)
        except NotFound:
            # Fallback to old system during migration
            return self.old_repo.get(user_id)

    def compare_consistency(self, limit: int = 1000) -> ConsistencyReport:
        """Compare data between old and new systems."""
        old_users = self.old_repo.list(limit=limit)
        new_users = self.new_repo.list(limit=limit)

        report = ConsistencyReport()
        old_map = {u.id: u for u in old_users}
        new_map = {u.id: u for u in new_users}

        for uid in set(old_map.keys()) | set(new_map.keys()):
            if uid not in old_map:
                report.only_in_new.append(uid)
            elif uid not in new_map:
                report.only_in_old.append(uid)
            elif old_map[uid] != new_map[uid]:
                report.mismatched.append((uid, old_map[uid], new_map[uid]))

        return report
```

### Schema Migration Coordination

```sql
-- Phase 1: Add new columns (old code ignores them)
ALTER TABLE users ADD COLUMN search_vector tsvector;
CREATE INDEX idx_users_search ON users USING GIN(search_vector);

-- Phase 2: Backfill new columns
UPDATE users SET search_vector = to_tsvector('english', name || ' ' || email);

-- Phase 3: Switch new code to use new columns
-- (deploy new service)

-- Phase 4: Drop old columns (after confirming new code works)
ALTER TABLE users DROP COLUMN old_search_field;
```

## One Source of Truth for Migrations

In polyglot systems, migration scripts can exist in multiple languages. This is a recipe for disaster.

### The Migration Registry

```python
# migration_registry.py — Single source of truth for migrations
from dataclasses import dataclass
from pathlib import Path
from typing import List

@dataclass
class Migration:
    version: str
    description: str
    up_sql: str
    down_sql: str
    applied_by: str  # Which language's service applied this
    checksum: str    # Verify migration hasn't been modified

class MigrationRegistry:
    """Central registry for all database migrations."""

    def __init__(self, db_connection):
        self.db = db_connection
        self._ensure_registry_table()

    def _ensure_registry_table(self):
        self.db.execute("""
            CREATE TABLE IF NOT EXISTS _migration_registry (
                version TEXT PRIMARY KEY,
                description TEXT,
                up_checksum TEXT,
                down_checksum TEXT,
                applied_by TEXT,
                applied_at TIMESTAMP DEFAULT NOW(),
                rollback_available BOOLEAN DEFAULT TRUE
            )
        """)

    def apply_migration(self, migration: Migration, applied_by: str):
        """Apply a migration and record it."""
        # Verify checksum
        if self._checksum_exists(migration.checksum):
            raise DuplicateMigrationError(
                f"Migration {migration.version} already applied"
            )

        # Execute up SQL
        self.db.execute(migration.up_sql)

        # Record in registry
        self.db.execute("""
            INSERT INTO _migration_registry
            (version, description, up_checksum, down_checksum, applied_by)
            VALUES (?, ?, ?, ?, ?)
        """, (migration.version, migration.description,
              migration.checksum, self._hash(migration.down_sql), applied_by))

    def rollback_safe(self, version: str) -> bool:
        """Check if rollback is safe (no dependent migrations applied after)."""
        result = self.db.execute("""
            SELECT COUNT(*) FROM _migration_registry
            WHERE version > ? AND rollback_available = FALSE
        """, (version,))
        return result.fetchone()[0] == 0
```

### Fusion.toml Migration Configuration

```toml
# Fusion.toml: migration management
[migrations]
# Single source of truth for all migrations
source_dir = "migrations/"
# Lock file prevents concurrent migrations
lock_file = ".fusion/migration.lock"
# Timeout for migration lock (5 minutes)
lock_timeout = "5m"

[migrations.registry]
# Central database for migration tracking
table = "_fusion_migrations"
# All migrations must go through this registry
enforce = true

[migrations.hooks]
# Run validation before applying
pre_apply = ["python -m fusion.migrations.validate"]
# Run after applying
post_apply = ["python -m fusion.migrations.verify"]
# Run on rollback
pre_rollback = ["python -m fusion.migrations.backup"]
```

## Strangler Fig Pattern

The Strangler Fig pattern is the gold standard for gradual migration. Named after the fig vine that grows around trees until it replaces them entirely.

### Implementation Guide

```
Step 1: Identify the boundary
  ┌─────────────────────────────────────┐
  │              API Gateway            │
  │         (Strangler Wrapper)         │
  └─────────────┬───────────────────────┘
                │
         ┌──────┴──────┐
         ▼             ▼
   ┌──────────┐  ┌──────────┐
   │  Old     │  │  (new)   │
   │  System  │  │  empty   │
   └──────────┘  └──────────┘

Step 2: Write new services one at a time
  ┌─────────────────────────────────────┐
  │              API Gateway            │
  └─────────────┬───────────────────────┘
                │
    ┌───────────┼───────────┐
    ▼           ▼           ▼
┌────────┐ ┌────────┐ ┌────────┐
│  Old   │ │ Users  │ │ (new)  │
│  Orders│ │ Service│ │ empty  │
└────────┘ └────────┘ └────────┘

Step 3: Repeat until old system has no traffic
  ┌─────────────────────────────────────┐
  │              API Gateway            │
  └─────────────┬───────────────────────┘
                │
    ┌───────────┼───────────┬───────────┐
    ▼           ▼           ▼           ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│ (old)  │ │ Users  │ │ Orders │ │ Pay    │
│ empty  │ │        │ │        │ │        │
└────────┘ └────────┘ └────────┘ └────────┘

Step 4: Remove old system
```

### Real-World Strangler Fig Example

```python
# gateway/strangler.py — Strangler Fig implementation
from fastapi import FastAPI, Request
import httpx

app = FastAPI()

# Service routing table
SERVICES = {
    "/api/v1/users": "http://user-service:8081",     # Migrated
    "/api/v1/orders": "http://order-service:8082",   # Migrated
    "/api/v1/payments": "http://payment-service:8083",  # Migrated
    "/api/v1/reports": "http://legacy-system:8080",  # Still on old
    "/api/v1/inventory": "http://legacy-system:8080", # Still on old
}

@app.api_route("/{path:path}", methods=["GET", "POST", "PUT", "DELETE"])
async def strangler_proxy(request: Request, path: str):
    """Route requests to appropriate service."""
    url = f"/{path}"

    # Find matching service
    target = None
    for prefix, service_url in SERVICES.items():
        if url.startswith(prefix):
            target = service_url
            break

    if not target:
        # Default to legacy system for unmigrated endpoints
        target = "http://legacy-system:8080"

    # Forward request
    async with httpx.AsyncClient() as client:
        response = await client.request(
            method=request.method,
            url=f"{target}{url}",
            headers=dict(request.headers),
            content=await request.body(),
        )

    return Response(
        content=response.content,
        status_code=response.status_code,
        headers=dict(response.headers),
    )
```

### Metrics for Migration Progress

```python
# migration_metrics.py — Track migration progress
from dataclasses import dataclass
from typing import Dict

@dataclass
class MigrationProgress:
    total_endpoints: int
    migrated_endpoints: int
    total_traffic_percentage: float  # % of traffic hitting new services
    days_since_start: int
    estimated_completion_days: int

def calculate_progress(services: Dict[str, dict]) -> MigrationProgress:
    total = len(services)
    migrated = sum(1 for s in services.values() if s["status"] == "migrated")
    traffic_new = sum(
        s["traffic_percentage"] for s in services.values()
        if s["status"] == "migrated"
    )

    return MigrationProgress(
        total_endpoints=total,
        migrated_endpoints=migrated,
        total_traffic_percentage=traffic_new,
        days_since_start=services["metadata"]["start_date"],
        estimated_completion_days=services["metadata"]["eta_days"],
    )
```

## Migration Checklist

Before starting any polyglot migration, verify:

```
□ All services have health checks
□ Rollback plan is tested and documented
□ Database migrations are backward-compatible
□ Monitoring covers both old and new systems
□ Feature flags control traffic routing
□ Consistency checks are in place
□ Team is trained on new system
□ Performance benchmarks are established
□ Cost comparison is documented
□ Timeline has buffer for unexpected issues
```

## Common Mistakes

### 1. Big Bang Migration

Migrating everything at once is the #1 cause of failed migrations. Strangler Fig exists for a reason.

### 2. Ignoring Database State

Migrating services without migrating the database leaves you with orphaned data.

### 3. Forgetting About Observability

During migration, you need logs from both old and new systems. Without correlation, debugging becomes impossible.

### 4. Underestimating Testing Effort

Every migrated service needs regression testing against the old system. This often doubles the testing workload.

### 5. No Rollback Plan

If the new service fails at 3 AM, you need to switch back to the old system in minutes, not hours.

## Summary

Polyglot exit strategy isn't a failure — it's maturation. The key principles:

1. **Migrate gradually** using Strangler Fig
2. **Keep rollback capability** at every step
3. **Single source of truth** for database migrations
4. **Monitor everything** during transition
5. **Measure progress** with clear metrics
6. **Document the decision** — future teams need to know why

The goal isn't to eliminate polyglot — it's to ensure the architecture serves the team, not the other way around.
