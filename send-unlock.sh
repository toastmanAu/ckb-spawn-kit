#!/bin/bash
RPC="http://192.168.68.134:8114/rpc"
LOCK_TX="0x1b6ba9f7119977e389f532c8b28fa7c7e874a689abe7bc0a09159a895bc65e9e"
BLOCK_HASH="0xb7dd98b97c55e53c597776d0ef490c05559b9d0564b5661b7a6efb101f21ef39"
TIMELOCK_MODULE="0xd927d1dc9764144620c1721c98a732ba193bf07463a18c2a4f09406dbe4d9237"

echo "=== Checking custom script resolution ==="
# Verify data_hash lookup works
DATA_RESULT=$(curl -s -X POST "$RPC" -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"get_cells","params":[{"script":{"code_hash":"0x7be578e4d420a6d88b3e6b1899e73b0677726667baa78f42e1ac2fbf1f39ef16","hash_type":"data","args":"0x"},"script_type":"lock"},"asc","0x0a"]}')
echo "data_hash lookup: $(echo "$DATA_RESULT" | python3 -c "import json,sys; r=json.load(sys.stdin).get('result',[]); print(f'{len(r)} cells found')" 2>/dev/null)"

# Send unlock tx
echo "=== Sending unlock transaction ==="
UNLOCK_TX='{"version":"0x0","cell_deps":[{"out_point":{"tx_hash":"'"$TIMELOCK_MODULE"'","index":"0x0"},"dep_type":"code"}],"header_deps":["'"$BLOCK_HASH"'"],"inputs":[{"since":"0x0","previous_output":{"tx_hash":"'"$LOCK_TX"'","index":"0x0"}}],"outputs":[{"capacity":"0x430e23400","lock":{"code_hash":"0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8","hash_type":"type","args":"0xef5be9fef2be33972398e150692fa987c3bfbaeb"},"type":null}],"outputs_data":["0x"],"witnesses":["0x"]}'

RESULT=$(curl -s -X POST "$RPC" -H "Content-Type: application/json" \
  -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"send_transaction\",\"params\":[$UNLOCK_TX,\"passthrough\"]}")
echo "$RESULT" | python3 -c "
import json,sys
r=json.load(sys.stdin)
if 'result' in r:
    print(f'SUCCESS! tx: {r[\"result\"]}')
    print('SPAWN COMPOSITION WORKS ON CKB TESTNET!')
elif 'error' in r:
    msg = r['error'].get('message','')
    print(f'Error: {msg[:500]}')
" 2>/dev/null || echo "Raw: $RESULT"
