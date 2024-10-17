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
    data.append(new_card)
    write_data(data)
    return jsonify(new_card), 201

@app.route('/cards/<int:card_id>', methods=['PUT'])
def update_card(card_id):
    updated_card = request.json
    data = read_data()
    if 0 <= card_id < len(data):
        data[card_id] = updated_card
        write_data(data)
        return jsonify(updated_card)
    return jsonify({'error': 'Card not found'}), 404

@app.route('/cards/<int:card_id>', methods=['DELETE'])
def delete_card(card_id):
    data = read_data()
    if 0 <= card_id < len(data):
        deleted_card = data.pop(card_id)
        write_data(data)
        return jsonify(deleted_card)
    return jsonify({'error': 'Card not found'}), 404

def start_admin_server():
    app.run(debug=True)

