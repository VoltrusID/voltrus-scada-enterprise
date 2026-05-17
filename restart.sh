#!/bin/bash
set -e
cd /Users/macbookpro/Documents/JoyoDigitama/voltrus/voltrus

echo "Stopping all..."
pkill -f "voltrus|modbus-sim|s7-sim|opcua-sim|enip-sim|dnp3-sim|bacnet-sim|mqtt-sim" 2>/dev/null || true
sleep 2

echo "Starting simulators..."
./target/release/modbus-sim --port 5020 > /tmp/voltrus-test/modbus1.log 2>&1 &
./target/release/modbus-sim --port 5021 > /tmp/voltrus-test/modbus2.log 2>&1 &
./target/release/s7-sim --port 10200 > /tmp/voltrus-test/s7.log 2>&1 &
./target/release/opcua-sim --host 127.0.0.1 --port 4840 > /tmp/voltrus-test/opcua.log 2>&1 &
./target/release/enip-sim --port 44818 > /tmp/voltrus-test/enip.log 2>&1 &
./target/release/dnp3-sim 20000 10 > /tmp/voltrus-test/dnp3.log 2>&1 &
./target/release/bacnet-sim --port 47809 --device 101 > /tmp/voltrus-test/bacnet.log 2>&1 &
./target/release/mqtt-sim --port 1884 > /tmp/voltrus-test/mqtt.log 2>&1 &

echo "Starting Voltrus..."
rm -f /tmp/voltrus-test/voltrus.db /tmp/voltrus-test/voltrus.log
./target/release/voltrus --import-activation test-activation.bin /tmp/voltrus-test/config.yaml
./target/release/voltrus /tmp/voltrus-test/config.yaml > /tmp/voltrus-test/voltrus.log 2>&1 &
sleep 4

PW=$(grep "ADMIN PASSWORD" /tmp/voltrus-test/voltrus.log | awk '{print $NF}')

# Quick device seed
BASE="http://localhost:3100"
curl -s -c /tmp/voltrus-test/cookies.txt "$BASE/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"$PW\"}" > /dev/null

curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/enip/devices" \
  -H "Content-Type: application/json" \
  -d '{"name":"CompactLogix","url":"127.0.0.1:44818","slot":0,"interval_ms":1000}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/enip/devices/1/tags" \
  -H "Content-Type: application/json" \
  -d '{"name":"Voltage_L1","tag_name":"Voltage_L1","data_type":"real","interval_ms":1000}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt -X POST "$BASE/api/v1/enip/devices/1/start" > /dev/null

curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/opcua/devices" \
  -H "Content-Type: application/json" \
  -d '{"name":"OPC_Meter","url":"opc.tcp://127.0.0.1:4840","security_policy":"None","security_mode":"None","interval_ms":1000}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/opcua/devices/1/nodes" \
  -H "Content-Type: application/json" \
  -d '{"node_id":"ns=2;s=Voltage_L1","name":"Voltage_L1","data_type":"float64"}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt -X POST "$BASE/api/v1/opcua/devices/1/start" > /dev/null

curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/s7/devices" \
  -H "Content-Type: application/json" \
  -d '{"name":"S7_1200","url":"127.0.0.1:10200","rack":0,"slot":1,"interval_ms":1000}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt "$BASE/api/v1/s7/devices/1/tags" \
  -H "Content-Type: application/json" \
  -d '{"tag_address":"DB1.DBD0","name":"voltage","data_type":"real","interval_ms":1000}' > /dev/null
curl -s -b /tmp/voltrus-test/cookies.txt -X POST "$BASE/api/v1/s7/devices/1/start" > /dev/null

sleep 5

echo ""
echo "╔════════════════════════════════════╗"
echo "║  http://localhost:3100             ║"
echo "║  admin / $PW  ║"
echo "╚════════════════════════════════════╝"
curl -s http://localhost:3100/api/v1/health | jq -r '.devices | to_entries[] | "  \(.key) → \(.value)"'
