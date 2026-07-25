import os
root = r'c:\Users\Matth\Downloads\Fusion v2.0 Vortex'

# 1. result.fu - trim generic class tail
p = os.path.join(root, 'stdlib', 'result.fu')
with open(p, 'r', encoding='utf-8') as f:
    lines = f.readlines()
out = []
for l in lines:
    if l.startswith('// Result<T, E> - Rust-style'):
        out.append('// NOTE: Generic Result<T, E> requires generics and first-class functions.\n')
        out.append('// Use concrete types: ResultIntString, ResultBoolString, ResultStringString.\n')
        break
    out.append(l)
with open(p, 'w', encoding='utf-8', newline='') as f:
    f.writelines(out)
print(f'result.fu: {len(out)} lines')

# 2. option.fu - trim generic class tail
p = os.path.join(root, 'stdlib', 'option.fu')
with open(p, 'r', encoding='utf-8') as f:
    lines = f.readlines()
out = []
for l in lines:
    if l.startswith('// Option<T> - Rust-style'):
        out.append('// NOTE: Generic Option<T> requires generics and first-class functions.\n')
        out.append('// Use concrete types: OptionInt, OptionBool, OptionString.\n')
        break
    out.append(l)
with open(p, 'w', encoding='utf-8', newline='') as f:
    f.writelines(out)
print(f'option.fu: {len(out)} lines')

# 3. iterator.fu - trim generic Iterator trait + class section
p = os.path.join(root, 'stdlib', 'iterator.fu')
with open(p, 'r', encoding='utf-8') as f:
    lines = f.readlines()
out = []
for l in lines:
    if l.startswith('// stdlib/iterator.fu - Iterator trait and utilities'):
        out.append('// NOTE: Generic Iterator<T> trait and collect() require generics.\n')
        out.append('// Use concrete RangeIterator and ReverseRangeIterator above.\n')
        break
    out.append(l)
with open(p, 'w', encoding='utf-8', newline='') as f:
    f.writelines(out)
print(f'iterator.fu: {len(out)} lines')

print('Done trimming generic class tails.')
