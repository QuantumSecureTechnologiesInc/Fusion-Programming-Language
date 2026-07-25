import os
root = r'c:\Users\Matth\Downloads\Fusion v2.0 Vortex'
NL = chr(10)

# === log.fu ===
p = os.path.join(root, 'stdlib', 'log.fu')
lines = open(p, 'r', encoding='utf-8').readlines()
out = []
for l in lines:
    s = l.strip()
    if l.startswith('class Logger {'):
        out.append('struct Logger {' + NL)
    elif s == 'level: Level;':
        out.append('    level: int,' + NL)
    elif s == 'file_path: string;':
        out.append('    file_path: string,' + NL)
    elif s == 'use_file: bool;':
        out.append('    use_file: bool,' + NL)
    elif s == 'file: fs::File;':
        out.append('    file_handle: int,' + NL)
    elif s.startswith('fn new(level:'):
        out.append('}' + NL + NL + 'impl Logger {' + NL)
        out.append('    fn new(level: int) -> Logger {' + NL)
    elif 'file: fs::File::open' in l:
        out.append('            file_handle: 0,' + NL)
    elif s.startswith('fn with_file(mut self'):
        out.append('    fn with_file(lg: Logger, path: string) -> Logger {' + NL)
    elif s.startswith('fn log(mut self, lvl:'):
        out.append('    fn log(lg: Logger, lvl: int, msg: string) {' + NL)
    elif 'if (lvl < self.level)' in l:
        out.append(l.replace('self.level', 'lg.level'))
    elif s.startswith('fn info(mut self'):
        out.append('    fn info(lg: Logger, msg: string) { Logger::log(lg, 1, msg); }' + NL)
    elif s.startswith('fn warn(mut self'):
        out.append('    fn warn(lg: Logger, msg: string) { Logger::log(lg, 2, msg); }' + NL)
    elif s.startswith('fn error(mut self'):
        out.append('    fn error(lg: Logger, msg: string) { Logger::log(lg, 3, msg); }' + NL)
    elif s.startswith('fn debug(mut self'):
        out.append('    fn debug(lg: Logger, msg: string) { Logger::log(lg, 0, msg); }' + NL)
    elif 'self.file_path' in l:
        out.append(l.replace('self.file_path', 'lg.file_path'))
    elif 'self.use_file' in l:
        out.append(l.replace('self.use_file', 'lg.use_file'))
    elif 'self.file' in l:
        out.append(l.replace('self.file', 'lg.file_handle'))
    elif 'self.' in l:
        out.append(l.replace('self.', 'lg.'))
    elif 'Logger::new(Level::Info)' in l:
        out.append(l.replace('Logger::new(Level::Info)', 'Logger::new(1)'))
    elif 'l.info(msg)' in l:
        out.append(l.replace('l.info(msg)', 'Logger::info(l, msg)'))
    else:
        out.append(l)
open(p, 'w', encoding='utf-8', newline='').writelines(out)
print(f'log.fu: {len(out)} lines')

# === http.fu ===
p = os.path.join(root, 'stdlib', 'http.fu')
lines = open(p, 'r', encoding='utf-8').readlines()
out = []
for l in lines:
    s = l.strip()
    if l.startswith('class Request {'):
        out.append('struct Request {' + NL)
    elif l.startswith('class Response {'):
        out.append('struct Response {' + NL)
    elif l.startswith('class SimpleServer {'):
        out.append('struct SimpleServer {' + NL)
    elif s == 'method: string;':
        out.append('    method: string,' + NL)
    elif s == 'path: string;':
        out.append('    path: string,' + NL)
    elif s == 'body: string;':
        out.append('    body: string,' + NL)
    elif s == 'status: int;':
        out.append('    status: int,' + NL)
    elif s == 'port: int;':
        out.append('    port: int,' + NL)
    elif s.startswith('socket:') and 'UdpSocket' in s:
        out.append('    socket_fd: int,' + NL)
    elif s.startswith('fn new(status:'):
        out.append('}' + NL + NL + 'impl Response {' + NL)
        out.append('    fn new(status: int, body: string) -> Response {' + NL)
    elif s.startswith('fn to_string(self)'):
        out.append('    fn to_string(r: Response) -> string {' + NL)
    elif 'self.status' in l:
        out.append(l.replace('self.status', 'r.status').replace('self.body', 'r.body'))
    elif 'self.body' in l:
        out.append(l.replace('self.body', 'r.body'))
    elif s.startswith('fn new(port:'):
        out.append('}' + NL + NL + 'impl SimpleServer {' + NL)
        out.append('    fn new(port: int) -> SimpleServer {' + NL)
    elif 'net::UdpSocket::bind' in l:
        out.append('        return SimpleServer { port: port, socket_fd: 0 };' + NL)
    elif s.startswith('fn handle_request('):
        out.append('    fn handle_request(req_str: string) -> Response {' + NL)
    elif s.startswith('fn serve(mut self'):
        out.append('    fn serve(sv: SimpleServer) {' + NL)
    elif 'self.port' in l:
        out.append(l.replace('self.port', 'sv.port'))
    else:
        out.append(l)
open(p, 'w', encoding='utf-8', newline='').writelines(out)
print(f'http.fu: {len(out)} lines')
