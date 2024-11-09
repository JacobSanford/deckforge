import threading

from flask import Flask, jsonify

from deckforge.blockchain.chain import Blockchain

app = Flask(__name__)
blockchain = Blockchain()
lock = threading.Lock()

@app.route('/blockchain', methods=['GET'])
def show_blockchain():
    return jsonify(blockchain.chain)

@app.route('/card/mint', methods=['GET'])
def mint_new_card():
    with lock:
        posted_address = '0x60333F6F01199ddFfAa9fdB1C7fD94E887ac7758'
        # priv_key = '6cb4891f108e0e22febe278bbb1d5aeae595f020c9c95b5b34e47ba02549f110'
        # publ_key = 'b94807660e37a8704f9235e3203b4b3af2f83311b2a4152766cec1c5f0f2fe511b62fbe70fa2a41b6839b672968593214c9d1d12d3331df551f9c6c3d42500fc'
        blockchain.execute_smart_contract('mint_card', blockchain, posted_address)
    return jsonify(blockchain.chain)

def start_api_server():
    app.run(host='0.0.0.0', port=5000)
