#!/usr/bin/python3
import sys
import json

data = input()
payload = json.loads(data)
response = json.dumps({"error": "debug error", "action": {"type": "Custom"}})
print(response)
