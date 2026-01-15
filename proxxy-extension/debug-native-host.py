#!/usr/bin/env python3
"""
Debug script for testing Proxxy Native Host
This helps diagnose connection issues between Chrome Extension and Native Host
"""

import sys
import json
import struct
import threading
import time
import os

def debug_native_host():
    """Simulate Proxxy native host for debugging"""
    print("=== Proxxy Native Host Debug Script ===")
    print(f"Process ID: {os.getpid()}")
    print(f"Script path: {os.path.abspath(__file__)}")
    print(f"Working directory: {os.getcwd()}")
    print("Waiting for Chrome Extension connection...")
    print("Send messages via stdin, receive via stdout")
    print("Press Ctrl+C to exit")
    print("=" * 50)
    
    def send_message(message):
        """Send message to Chrome Extension"""
        encoded = json.dumps(message).encode('utf-8')
        message_length = struct.pack('I', len(encoded))
        sys.stdout.buffer.write(message_length + encoded)
        sys.stdout.buffer.flush()
    
    def receive_message():
        """Receive message from Chrome Extension"""
        try:
            # Read 4-byte length header
            raw_length = sys.stdin.buffer.read(4)
            if len(raw_length) == 0:
                return None
            
            message_length = struct.unpack('I', raw_length)[0]
            
            # Read the message
            message = sys.stdin.buffer.read(message_length).decode('utf-8')
            return json.loads(message)
        except Exception as e:
            print(f"Error receiving message: {e}")
            return None
    
    def handle_message(message):
        """Handle incoming message from extension"""
        print(f"Received: {message}")
        
        response = {
            "id": message.get("id", "unknown"),
            "success": True,
            "data": {
                "message": "Debug response",
                "timestamp": time.time(),
                "received_message": message
            }
        }
        
        # Special handling for ping
        if message.get("action") == "ping":
            response["data"] = {"pong": True, "timestamp": time.time()}
        
        # Special handling for system messages
        if message.get("module") == "system":
            response["data"] = {"status": "debug_mode", "version": "debug-0.1.0"}
        
        print(f"Sending: {response}")
        send_message(response)
    
    # Main loop
    try:
        while True:
            message = receive_message()
            if message is None:
                break
            handle_message(message)
    
    except KeyboardInterrupt:
        print("\nShutting down debug host...")
    except Exception as e:
        print(f"Fatal error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    debug_native_host()