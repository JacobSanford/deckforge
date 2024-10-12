import os
from flask import Flask, jsonify
from deckforge.blockchain.core import Blockchain
from deckforge.card.minter import CardMinter

app = Flask(__name__)
blockchain = Blockchain()

@app.route('/blockchain', methods=['GET'])
def show_blockchain():
    return jsonify(blockchain.chain)

@app.route('/card/mint', methods=['GET'])
def mint_new_card():
    minter = CardMinter('/home/jsanford/gitDev/deckforge/data/card_metadata.json')
    new_card = minter.mint_card()
    blockchain.mint_card_to_blockchain(new_card)
    return jsonify(new_card)

def start_api_server():
    app.run(host='0.0.0.0', port=5000)
