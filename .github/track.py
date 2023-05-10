from datetime import datetime
import json
import os
import re
import requests
import subprocess
import sys


def to_cmp(x): return [int(y) for y in x.split('.')]


rts = {
    'darwinia': ['crab', 'darwinia'],
    'darwinia-common': ['pangolin', 'pangoro'],
}
token = os.environ['AUTH']
repo = sys.argv[1]
new_rts = []

resp = requests.get(
    f'https://api.github.com/repos/darwinia-network/{repo}/releases/latest',
    headers={'Authorization': f'token {token}'})
data = json.loads(resp.text)
tag = data['tag_name']
v = ['0.0.0', '0']

for rt in rts[repo]:
    rt_folder = f'{rt}/wasms'

    if not os.path.isdir(rt_folder):
        os.makedirs(rt_folder)

    fs = os.listdir(f'{rt}/wasms')

    for f in fs:
        v_ = f.split('-v', 1)[1].split('-tracing', 1)[0].split('-', 1)

        main_v = to_cmp(v[0])
        main_v_ = to_cmp(v_[0])

        if main_v < main_v_:
            v[0] = v_[0]

            if len(v_) > 1:
                v[1] = v_[1]
            else:
                v[1] = '0'
        elif main_v == main_v_:
            if len(v_) > 1:
                sub_v = int(v[1])
                sub_v_ = int(v_[1])

                if sub_v < sub_v_:
                    v[1] = v_[1]

    if v[1] == '0':
        tag_ = f'v{v[0]}'
    else:
        tag_ = f'v{v[0]}-{v[1]}'

    if re.search(r'v\d+.\d+.\d+(-\d+)?', tag).group() != tag_:
        subprocess.run(['runtime-overrides', '-r', rt, '-t', tag, '-o', '.'])
        new_rts.append(f'{rt}-{tag}')

if new_rts:
    with open('CHANGELOG', 'a+') as f:
        rts = ", ".join(new_rts)
        date = datetime.now().strftime("%m/%d/%Y %H:%M:%S")

        f.write(f'Generated {rts} - {date}\n')
