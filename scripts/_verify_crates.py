import os, subprocess

crate_dir = 'crates'
fuc = 'bin/fuc.exe'
results = {'pass': [], 'fail': []}
count = 0

for root, dirs, files in os.walk(crate_dir):
    for f in files:
        if f.endswith('.fu'):
            path = os.path.join(root, f)
            count += 1
            if count > 50:
                break
            cmd = [fuc, path, '--lib']
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
                if r.returncode == 0:
                    results['pass'].append(path)
                else:
                    err = (r.stderr + r.stdout)[:100].replace('\n', ' ').strip()
                    results['fail'].append(f'{path}: {err}')
            except:
                results['fail'].append(f'{path}: TIMEOUT')
    if count > 50:
        break

print(f'Checked {count} files, PASS={len(results["pass"])}, FAIL={len(results["fail"])}')
for p in results['pass']:
    print(f'  OK  {p}')
for p in results['fail']:
    print(f'  FAIL {p}')