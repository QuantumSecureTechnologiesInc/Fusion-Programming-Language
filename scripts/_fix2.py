import os
root = r'c:\Users\Matth\Downloads\Fusion v2.0 Vortex'

# json.fu
p = os.path.join(root, 'stdlib', 'json.fu')
lines = open(p, 'r', encoding='utf-8').readlines()
out = []
for l in lines:
    s = l.strip()
    if l.startswith('class JsonValue {'):
        out.append('struct JsonValue {' + chr(10))
    elif l.startswith('class ConfigParser {'):
        out.append('struct ConfigParser {' + chr(10))
    elif s.endswith(';') and ':' in s and not s.startswith('//') and not s.startswith('return') and not s.startswith('if'):
        out.append(l.rstrip().rstrip(';') + ',' + chr(10))
    elif s.startswith('fn new(src:'):
        out.append('}' + chr(10) + chr(10) + 'impl ConfigParser {' + chr(10))
        out.append('    fn new(src: string) -> ConfigParser {' + chr(10))
    elif s == 'fn parse_string(mut self) -> string {':
        out.append('    fn parse_string(cp: ConfigParser) -> string {' + chr(10))
    elif s == 'fn get_key(mut self, key: string) -> string {':
        out.append('    fn get_key(cp: ConfigParser, key: string) -> string {' + chr(10))
    elif 'self.pos' in l or 'self.source' in l:
        out.append(l.replace('self.pos', 'cp.pos').replace('self.source', 'cp.source'))
    else:
        out.append(l)
open(p, 'w', encoding='utf-8', newline='').writelines(out)
print(f'json.fu: {len(out)} lines')

# linkedlist.fu
p = os.path.join(root, 'stdlib', 'linkedlist.fu')
lines = open(p, 'r', encoding='utf-8').readlines()
keep = []
for l in lines:
    if l.startswith('// Doubly-linked list implementation with generic'):
        keep.append('// NOTE: Generic LinkedList<T> requires generics support.' + chr(10))
        keep.append('// Use the concrete LinkedListInt above.' + chr(10))
        break
    keep.append(l)
open(p, 'w', encoding='utf-8', newline='').writelines(keep)
print(f'linkedlist.fu: {len(keep)} lines')
