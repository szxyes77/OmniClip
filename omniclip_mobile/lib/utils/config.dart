import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class Config {
  static Config? _instance;
  static Config get instance => _instance ??= Config._();

  Config._();

  String serverIp = '192.168.0.107';
  int serverPort = 18911;

  String get baseUrl => 'http://$serverIp:$serverPort';

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    serverIp = prefs.getString('server_ip') ?? '192.168.0.107';
    serverPort = prefs.getInt('server_port') ?? 18911;
  }

  Future<void> save() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('server_ip', serverIp);
    await prefs.setInt('server_port', serverPort);
  }

  Future<void> updateBaseUrl(String ip, int port) async {
    serverIp = ip;
    serverPort = port;
    await save();
  }
}
