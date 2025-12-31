#!/usr/bin/env python3
"""
Simple benchmark script for ToonStore RESP server
Measures throughput and latency for basic operations
"""

import socket
import time
import statistics

def send_command(sock, *args):
    """Send a RESP command"""
    cmd = f"*{len(args)}\r\n"
    for arg in args:
        arg_bytes = str(arg).encode('utf-8')
        cmd += f"${len(arg_bytes)}\r\n{arg_bytes.decode()}\r\n"
    sock.sendall(cmd.encode('utf-8'))

def read_response(sock):
    """Read a RESP response"""
    data = sock.recv(1024)
    return data.decode('utf-8', errors='ignore')

def benchmark_operation(host, port, operation, iterations=10000):
    """Benchmark a specific operation"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    
    latencies = []
    start_time = time.time()
    
    for i in range(iterations):
        op_start = time.time()
        
        if operation == "PING":
            send_command(sock, "PING")
        elif operation == "SET":
            send_command(sock, "SET", f"key{i}", f"value{i}")
        elif operation == "GET":
            send_command(sock, "GET", str(i % 1000))
        elif operation == "DEL":
            send_command(sock, "DEL", str(i % 1000))
        
        response = read_response(sock)
        
        op_end = time.time()
        latencies.append((op_end - op_start) * 1000000)  # microseconds
    
    end_time = time.time()
    sock.close()
    
    total_time = end_time - start_time
    ops_per_sec = iterations / total_time
    
    return {
        'operation': operation,
        'iterations': iterations,
        'total_time': total_time,
        'ops_per_sec': ops_per_sec,
        'latency_avg': statistics.mean(latencies),
        'latency_p50': statistics.median(latencies),
        'latency_p95': sorted(latencies)[int(len(latencies) * 0.95)],
        'latency_p99': sorted(latencies)[int(len(latencies) * 0.99)],
    }

def main():
    HOST = '127.0.0.1'
    PORT = 6380
    ITERATIONS = 10000
    
    print("╔═══════════════════════════════════════════════════════════════╗")
    print("║         ToonStore RESP Server Benchmark                     ║")
    print("╚═══════════════════════════════════════════════════════════════╝\n")
    
    print(f"Configuration:")
    print(f"  Host: {HOST}:{PORT}")
    print(f"  Iterations: {ITERATIONS}")
    print(f"\nRunning benchmarks...\n")
    
    operations = ['PING', 'SET', 'GET']
    results = []
    
    for op in operations:
        print(f"Benchmarking {op}...", end=' ', flush=True)
        result = benchmark_operation(HOST, PORT, op, ITERATIONS)
        results.append(result)
        print(f"✓ {result['ops_per_sec']:.0f} ops/sec")
    
    print("\n" + "="*70)
    print("RESULTS")
    print("="*70 + "\n")
    
    for result in results:
        print(f"{result['operation']:10} │ {result['ops_per_sec']:>10,.0f} ops/sec │ "
              f"Avg: {result['latency_avg']:>6.1f} µs │ "
              f"P50: {result['latency_p50']:>6.1f} µs │ "
              f"P95: {result['latency_p95']:>6.1f} µs │ "
              f"P99: {result['latency_p99']:>6.1f} µs")
    
    print("\n" + "="*70)
    print(f"\n✅ Benchmark complete!")
    
    # Calculate overall performance
    total_ops = sum(r['ops_per_sec'] for r in results)
    avg_ops = total_ops / len(results)
    print(f"\n📊 Average throughput: {avg_ops:,.0f} ops/sec")
    
    # Check kill switch
    if avg_ops < 30000:
        print(f"\n⚠️  WARNING: Performance below 30k ops/sec kill switch!")
        print(f"   Recommendation: Ship embedded library only")
    elif avg_ops >= 50000:
        print(f"\n🎉 SUCCESS: Exceeded 50k ops/sec target!")
    else:
        print(f"\n✓  PASS: Above 30k ops/sec kill switch")

if __name__ == '__main__':
    main()
