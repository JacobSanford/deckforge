from flask import Flask, request, jsonify, render_template, send_from_directory
import json
import os
from werkzeug.utils import secure_filename

app = Flask(__name__)
DATA_FILE = '/home/jsanford/gitDev/deckforge/data/card_metadata.json'
UPLOAD_FOLDER = '/home/jsanford/gitDev/deckforge/data/images'
app.config['UPLOAD_FOLDER'] = UPLOAD_FOLDER

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
    new_card = request.form.to_dict()
    image_file = request.files.get('cardImage')
    if image_file:
        filename = secure_filename(image_file.filename)
        image_path = os.path.join(app.config['UPLOAD_FOLDER'], filename)
        image_file.save(image_path)
        new_card['image_path'] = f'/data/images/{filename}'

    data = read_data()
    new_card_id = max([card['id'] for card in data], default=0) + 1
    new_card['id'] = new_card_id
    data.append(new_card)
    write_data(data)
    return jsonify(new_card), 201

@app.route('/cards/<int:card_id>', methods=['GET'])
def get_card(card_id):
    data = read_data()
    for card in data:
        if card['id'] == card_id:
            return jsonify(card)
    return jsonify({'error': 'Card not found'}), 404

@app.route('/cards/<int:card_id>', methods=['PUT'])
def update_card(card_id):
    updated_card = request.form.to_dict()
    image_file = request.files.get('cardImage')
    if image_file:
        filename = secure_filename(image_file.filename)
        image_path = os.path.join(app.config['UPLOAD_FOLDER'], filename)
        image_file.save(image_path)
        updated_card['image_path'] = f'/data/images/{filename}'

    data = read_data()
    for card in data:
        if card['id'] == card_id:
            card.update(updated_card)
            write_data(data)
            return jsonify(card)
    return jsonify({'error': 'Card not found'}), 404

@app.route('/cards/<int:card_id>', methods=['DELETE'])
def delete_card(card_id):
    data = read_data()
    card_to_delete = next((card for card in data if card['id'] == card_id), None)
    if not card_to_delete:
        return jsonify({'error': 'Card not found'}), 404

    data.remove(card_to_delete)
    write_data(data)
    return jsonify(card_to_delete)

@app.route('/uploads/<filename>')
def uploaded_file(filename):
    return send_from_directory(app.config['UPLOAD_FOLDER'], filename)

def start_admin_server():
    app.run(debug=True)

