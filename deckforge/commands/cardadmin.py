from flask import Flask, request, jsonify, render_template
import json
import os

app = Flask(__name__)
DATA_FILE = '/home/jsanford/gitDev/deckforge/data/card_metadata.json'

def read_data():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE, 'r') as file:
            return json.load(file)
    return []

def write_data(data):
    with open(DATA_FILE, 'w') as file:
        json.dump(data, file, indent=4)

@app.route('/')
def index():
    data = read_data()
    return render_template('index.html', cards=data)

@app.route('/cards', methods=['GET'])
def get_cards():
    data = read_data()
    return jsonify(data)

@app.route('/cards', methods=['POST'])
def add_card():
    new_card = request.json
    data = read_data()
    for card in data:
        if int(card['id']) > int(new_card['id'] or 0):
            new_card['id'] = str(int(card['id']) + 1)
    data.append(new_card)
    write_data(data)
    return jsonify(new_card), 201

@app.route('/cards/<string:card_id>', methods=['PUT'])
def update_card(card_id):
    updated_card = request.json
    data = read_data()
    for card in data:
        if card['id'] == card_id:
            card.update(updated_card)
            write_data(data)
            return jsonify(card)
    return jsonify({'error': 'Card not found'}), 404

@app.route('/cards/<string:card_id>', methods=['DELETE'])
def delete_card(card_id):
    data = read_data()
    card_to_delete = {}
    for card in data:
        if card['id'] == card_id:
            card_to_delete = card
    if card_to_delete == {}:
        return jsonify({'error': 'Card not found'}), 404

    data.remove(card_to_delete)
    write_data(data)
    return jsonify(card_to_delete)

def start_admin_server():
    app.run(debug=True)

