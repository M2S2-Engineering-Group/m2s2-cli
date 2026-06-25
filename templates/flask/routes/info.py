from flask import Blueprint, jsonify

info_bp = Blueprint("info", __name__)


@info_bp.get("/info")
def info():
    return jsonify({"service": "api", "version": "1.0.0"})
