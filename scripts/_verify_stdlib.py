import subprocess
import os

stdlib_dir = 'stdlib'
fuc = 'bin/fuc.exe'
results = {'pass': [], 'fail': []}

for root, dirs, files in os.walk(stdlib_dir):
    for f in files:
        if f.endswith('.fu'):
            path = os.path.join(root, f)
            cmd = [fuc, path, '--lib']
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                if r.returncode == 0:
                    results['pass'].append(path)
                else:
                    err = (r.stderr + r.stdout)[:150].replace('\n', ' ').strip()
                    results['fail'].append(f'{path}: {err}')
            except Exception as e:
                results['fail'].append(f'{path}: ERROR {e}')

print(f'=== PASS: {len(results["pass"])} ===')
for p in results['pass']:
    print(f'  OK  {p}')
print(f'=== FAIL: {len(results["fail"])} ===')
for p in results['fail']:
    print(f'  FAIL {p}')