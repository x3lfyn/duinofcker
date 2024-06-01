import requests
from time import sleep, time_ns
from socket import socket
from hashlib import sha1

class Client:
    """
    Class helping to organize socket connections
    """
    def connect(pool: tuple):
        global s
        s = socket()
        s.settimeout(20)
        s.connect((pool))

    def send(msg: str):
        sent = s.sendall(str(msg).encode("UTF8"))
        return sent

    def recv(limit: int = 128):
        data = s.recv(limit).decode("UTF8").rstrip("\n")
        return data

    def fetch_pool(retry_count=1):
        """
        Fetches the best pool from the /getPool API endpoint
        """

        while True:
            if retry_count > 60:
                retry_count = 60

            try:
                response = requests.get(
                    "https://server.duinocoin.com/getPool",
                    timeout=20).json()

                if response["success"] == True:
                    print(f"connecting to: {response['name']}")

                    NODE_ADDRESS = response["ip"]
                    NODE_PORT = response["port"]

                    return (NODE_ADDRESS, NODE_PORT)

                elif "message" in response:
                    print(f"warning: {respone['message']}")

                else:
                    raise Exception("no response - IP ban or connection error")
            except Exception as e:
                print(f"error: {e}")
            sleep(retry_count * 2)
            retry_count += 1

def algo(last_h, exp_h, diff):
    start_time = time_ns()

    base = sha1(last_h.encode('ascii'))
    for nonce in range(100 * diff + 1):
        temp = base.copy()
        temp.update(str(nonce).encode('ascii'))
        dres = temp.hexdigest()

        if dres == exp_h:
            end_time = time_ns()
            elapsed = end_time - start_time
            hashrate = 1e9 * nonce / elapsed

            return [nonce, hashrate]

    return [0, 0]

pool = Client.fetch_pool()
conn = Client.connect(pool)
pool_version = Client.recv(5)
print(f"pool vers: {pool_version}")

print(pool)
Client.send("MOTD")
motd = Client.recv(512)
print(f"motd: {motd}")

for i in range(10000):

    key = "lalala"
    Client.send(f'JOB,nyaaaa,AVR,{key}')

    job = Client.recv().split(',')
    print(f"recived job: {job}")

    res = algo(job[0], job[1], int(job[2]))

    print(f"res: {res}")

    res[1] = "100"

    sleep(8)

    Client.send(f"{res[0]},{res[1]},Official AVR Miner 4.0,0,DUCOID751353c367eb4d36")

    res = Client.recv()
    print(f"res: {res}")
