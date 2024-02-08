from datetime import datetime
import subprocess
import sys

tag = sys.argv[1]
runtime = sys.argv[2]

subprocess.run([
    'rtor',
    '-g',
    'https://github.com/darwinia-network/darwinia',
    '-t',
    tag,
    '-m',
    f'runtime/{runtime}/Cargo.toml',
    '-r',
    runtime,
    '-o',
    '.'
]).check_returncode()

with open('CHANGELOG', 'a+') as f:
    f.write(f'Generated {tag} at {datetime.now().strftime("%m/%d/%Y %H:%M:%S")}\n')
