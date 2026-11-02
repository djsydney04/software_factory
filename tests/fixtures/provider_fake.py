#!/usr/bin/env python3
"""Deterministic external-process substitute for Codex; never performs inference."""
import hashlib, json, os, pathlib, signal, subprocess, sys, time

cwd = pathlib.Path.cwd()
script_dir = pathlib.Path(__file__).resolve().parent
scenario = (script_dir / 'scenario').read_text().strip()
thread_id = ('fixture-' + hashlib.sha256(str(cwd).encode()).hexdigest()[:12]) if scenario.startswith('factory') else 'fixture-thread'
turn_id = 'fixture-turn'
overrides = {}
for index, argument in enumerate(sys.argv[:-1]):
    if argument == '-c':
        key, value = sys.argv[index + 1].split('=', 1)
        overrides[key] = value
features = {key.removeprefix('features.'): value == 'true'
            for key, value in overrides.items() if key.startswith('features.')}

def emit(value, partial=False):
    data = json.dumps(value, separators=(',', ':')) + '\n'
    if partial:
        for start in range(0, len(data), 3):
            sys.stdout.write(data[start:start+3]); sys.stdout.flush()
    else:
        sys.stdout.write(data); sys.stdout.flush()

def reply(i, result):
    emit({'id': i, 'result': result}, scenario == 'partial')

def started():
    emit({'method': 'turn/started', 'params': {'threadId': thread_id, 'turn': {'id': turn_id, 'status': 'inProgress', 'items': []}}})

def terminal(status='completed'):
    if scenario.startswith('factory'):
        history_path = script_dir / ('history-' + thread_id + '.json')
        history = json.loads(history_path.read_text()) if history_path.exists() else []
        for entry in history:
            if entry['id'] == turn_id: entry['status'] = status
        history_path.write_text(json.dumps(history))
    emit({'method': 'turn/completed', 'params': {'threadId': thread_id, 'turn': {'id': turn_id, 'status': status, 'items': []}}})

for line in sys.stdin:
    request = json.loads(line)
    method, params, i = request.get('method'), request.get('params', {}), request.get('id')
    if method:
        with ((script_dir / ('methods-' + thread_id)) if scenario.startswith('factory') else (cwd / 'methods')).open('a') as output: output.write(method + '\n')
    if method == 'initialize':
        reply(i, {'userAgent': 'codex_cli_rs/0.159.3 (fixture)', 'platformOs': 'fixture', 'platformFamily': 'unix', 'codexHome': '/not-returned'})
    elif method == 'initialized':
        pass
    elif method == 'config/read':
        effective = dict(features)
        if scenario == 'ignore_policy': effective['apps'] = True
        reply(i, {'config': {'features': effective, 'web_search': 'disabled',
            'mcp_servers': {'fixture_external': {'enabled': overrides.get('mcp_servers.fixture_external.enabled') != 'false'}}}})
    elif method == 'skills/list':
        reply(i, {'data': [{'cwd': str(cwd), 'errors': [], 'skills': [{'path': str(cwd / 'fixture-skill/SKILL.md'),
            'enabled': 'enabled=false' not in overrides.get('skills.config', ''), 'scope': 'user'}]}]})
    elif method == 'account/read':
        reply(i, {'account': {'type': 'chatgpt'}})
    elif method == 'model/list':
        reply(i, {'data': [{'id': 'gpt-6.1-sol', 'model': 'gpt-6.1-sol', 'displayName': 'Fixture Model', 'isDefault': True,
            'defaultReasoningEffort': 'medium', 'supportedReasoningEfforts': [
                {'reasoningEffort': effort, 'description': effort} for effort in ['low', 'medium', 'high']]}], 'nextCursor': None})
    elif method in ['thread/start', 'thread/resume']:
        if method == 'thread/resume': thread_id = params['threadId']
        response = {'thread': {'id': thread_id}, 'cwd': params['cwd'], 'model': params['model'], 'modelProvider': 'openai',
            'reasoningEffort': params['config']['model_reasoning_effort'], 'approvalPolicy': 'never', 'approvalsReviewer': 'user',
            'sandbox': {'type': 'workspaceWrite', 'writableRoots': params['config']['sandbox_workspace_write']['writable_roots'],
                'networkAccess': params['config']['sandbox_workspace_write']['network_access'], 'excludeSlashTmp': True,
                'excludeTmpdirEnvVar': True} if params['sandbox'] == 'workspace-write' else {'type': 'readOnly', 'networkAccess': False}}
        if scenario == 'extra_root': response['sandbox']['writableRoots'].append('/outside-owned-worktree')
        if scenario == 'network_expanded': response['sandbox']['networkAccess'] = True
        if scenario == 'tmp_expanded': response['sandbox']['excludeSlashTmp'] = False
        reply(i, response)
    elif method == 'thread/turns/list':
        history_path = script_dir / ('history-' + thread_id + '.json')
        history = json.loads(history_path.read_text()) if scenario.startswith('factory') and history_path.exists() else [{'id': 'fixture-turn', 'status': 'completed', 'items': []}]
        reply(i, {'data': list(reversed(history)), 'nextCursor': None})
    elif method == 'turn/start':
        count = (script_dir / ('turn-count-' + thread_id)) if scenario.startswith('factory') else (cwd / 'turn-start-count')
        count.write_text(str(int(count.read_text()) + 1) if count.exists() else '1')
        if scenario.startswith('factory'):
            turn_id = 'fixture-turn-' + count.read_text()
            history_path = script_dir / ('history-' + thread_id + '.json')
            history = json.loads(history_path.read_text()) if history_path.exists() else []
            history.append({'id': turn_id, 'status': 'inProgress', 'items': []})
            history_path.write_text(json.dumps(history))
        emit({'method': 'future/notification', 'params': {'ignored': True}})
        started()
        if scenario != 'lost_start_eof': reply(i, {'turn': {'id': turn_id, 'status': 'inProgress', 'items': []}})
        if scenario.startswith('factory'):
            if not features.get('shell_tool'):
                plan = {'action': 'wait', 'reason': 'Fixture is waiting for a clearer request.'} if scenario == 'factory_wait' else {
                    'action': 'plan', 'title': 'Set fixture value to ready', 'description': 'Set value.txt to ready and verify with check.py.',
                    'acceptance': ['value.txt contains ready followed by a newline', 'check.py exits zero'],
                    'checks': [{'program': '/usr/bin/python3', 'args': ['check.py'], 'cwd': '.', 'timeoutSeconds': 5}]}
                emit({'method': 'item/agentMessage/delta', 'params': {'threadId': thread_id, 'turnId': turn_id, 'delta': json.dumps(plan)}})
                terminal(); continue
            if scenario in ['factory_slow', 'factory_ack_only']: continue
            assert (cwd / 'value.txt').exists() and (cwd / 'check.py').exists(), 'factory fixture must use the disposable fixture repository'
            (cwd / 'value.txt').write_text('wrong\n' if scenario == 'factory_failed_check' else 'ready\n')
            subprocess.run(['git', 'add', '--', 'value.txt'], cwd=cwd, check=True, capture_output=True)
            subprocess.run(['git', '-c', 'user.name=Factory Fixture', '-c', 'user.email=factory@fixture.invalid',
                'commit', '-m', 'Set fixture value'], cwd=cwd, check=True, capture_output=True)
            emit({'method': 'item/agentMessage/delta', 'params': {'threadId': thread_id, 'turnId': turn_id, 'delta': 'Updated and committed value.txt.'}})
            terminal(); continue
        if scenario in ['eof', 'lost_start_eof']: sys.exit(0)
        if scenario == 'truncated': sys.stdout.write('{"partial":'); sys.stdout.flush(); sys.exit(0)
        if scenario == 'malformed': sys.stdout.write('not-json\n'); sys.stdout.flush(); sys.exit(0)
        if scenario == 'oversized': sys.stdout.write('x' * (1024 * 1024 + 1)); sys.stdout.flush(); continue
        if scenario == 'stderr_flood': os.write(2, b'x' * (2 * 1024 * 1024))
        if scenario in ['descendant', 'descendant_detached']:
            child = subprocess.Popen([sys.executable, '-c', 'import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);time.sleep(3600)'],
                start_new_session=scenario == 'descendant_detached')
            (cwd / 'descendant-pid').write_text(str(child.pid))
        if scenario in ['interrupt', 'interrupt_ack_only', 'steer', 'deadline', 'descendant', 'descendant_detached']: continue
        if scenario == 'usage':
            emit({'method': 'thread/tokenUsage/updated', 'params': {'threadId': 'fixture-thread', 'turnId': 'fixture-turn',
                'tokenUsage': {'last': {'outputTokens': 500, 'totalTokens': 600}}}}); continue
        if scenario == 'approval':
            emit({'id': 'approval-1', 'method': 'item/commandExecution/requestApproval', 'params': {'threadId': 'fixture-thread',
                'turnId': 'fixture-turn', 'itemId': 'command-1', 'command': 'fixture command', 'cwd': str(cwd)}}); continue
        if scenario == 'wrong_terminal':
            emit({'method': 'turn/completed', 'params': {'threadId': 'fixture-thread', 'turn': {'id': 'another-turn', 'status': 'completed'}}}); continue
        emit({'method': 'item/agentMessage/delta', 'params': {'threadId': 'fixture-thread', 'turnId': 'fixture-turn', 'delta': 'fixture result'}}, scenario == 'partial')
        terminal('failed' if scenario == 'failure' else 'completed')
    elif method == 'turn/interrupt':
        reply(i, {})
        if scenario not in ['interrupt_ack_only', 'factory_ack_only', 'deadline', 'descendant', 'descendant_detached']: terminal('interrupted')
    elif method == 'turn/steer':
        if params['expectedTurnId'] != turn_id:
            emit({'id': i, 'error': {'code': -32602, 'message': 'wrong active turn'}})
        else:
            reply(i, {'turnId': turn_id}); terminal()
    elif method == 'command/exec':
        assert params['sandboxPolicy']['type'] == 'workspaceWrite'
        assert params['sandboxPolicy']['networkAccess'] is False
        assert params['sandboxPolicy']['excludeSlashTmp'] is True
        assert params['timeoutMs'] > 0 and params['outputBytesCap'] == 65536
        if scenario.startswith('factory'):
            try:
                result = subprocess.run(params['command'], cwd=params['cwd'], capture_output=True, text=True,
                    timeout=params['timeoutMs'] / 1000)
                reply(i, {'exitCode': result.returncode, 'stdout': result.stdout[:65536], 'stderr': result.stderr[:65536]})
            except subprocess.TimeoutExpired:
                reply(i, {'exitCode': 124, 'stdout': '', 'stderr': 'Fixture command timed out'})
        else:
            assert params['command'] == ['fixture-check', 'literal $(not-a-shell)']
            reply(i, {'exitCode': 7, 'stdout': 'observed check output', 'stderr': ''})
    elif method is None and i == 'approval-1':
        assert request['result']['decision'] == 'decline'
        terminal('failed')
    elif i is not None:
        emit({'id': i, 'error': {'code': -32601, 'message': 'unknown fixture method'}})
