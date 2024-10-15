import os
from flask import Flask, jsonify
from deckforge.blockchain.core import Blockchain

app = Flask(__name__)
blockchain = Blockchain()

@app.route('/blockchain', methods=['GET'])
def show_blockchain():
    return jsonify(blockchain.chain)

@app.route('/card/mint', methods=['GET'])
def mint_new_card():
    blockchain.execute_smart_contract('mint_card', blockchain, 'jsanford')
    return jsonify(blockchain.chain)

def start_api_server():
    app.run(host='0.0.0.0', port=5000)
