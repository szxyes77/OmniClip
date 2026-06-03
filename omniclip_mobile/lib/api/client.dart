import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:io' show Socket;
import 'package:flutter/foundation.dart';

class ClipboardRecord {
  final String id;
  final String recordType;
  final String content;
  final String title;
  final String contentHash;
  final String createdAt;
  final String updatedAt;
  final String lastUsedAt;
  final int copyCount;
  final int starCount;
  final bool isStarred;
  final String? thumbnailPath;
  final List<String>? tags;
  final String? sourceApp;

  ClipboardRecord({
    required this.id,
    required this.recordType,
    required this.content,
    required this.title,
    required this.contentHash,
    required this.createdAt,
    required this.updatedAt,
    required this.lastUsedAt,
    required this.copyCount,
    required this.starCount,
    required this.isStarred,
    this.thumbnailPath,
    this.tags,
    this.sourceApp,
  });

  factory ClipboardRecord.fromJson(Map<String, dynamic> json) {
    return ClipboardRecord(
      id: json['id']?.toString() ?? '',
      recordType: json['record_type']?.toString() ?? 'text',
      content: json['content']?.toString() ?? '',
      title: json['title']?.toString() ?? '',
      contentHash: json['content_hash']?.toString() ?? '',
      createdAt: json['created_at']?.toString() ?? '',
      updatedAt: json['updated_at']?.toString() ?? '',
      lastUsedAt: json['last_used_at']?.toString() ?? '',
      copyCount: int.tryParse(json['copy_count']?.toString() ?? '0') ?? 0,
      starCount: int.tryParse(json['star_count']?.toString() ?? '0') ?? 0,
      isStarred: json['is_starred'] == true || json['is_starred'] == 1,
      thumbnailPath: json['thumbnail_path'],
      tags: (json['tags'] as List<dynamic>?)?.map((e) => e.toString()).toList(),
      sourceApp: json['source_app']?.toString(),
    );
  }
}

class OmniClipApi {
  final String baseUrl;

  OmniClipApi(this.baseUrl);

  Future<List<ClipboardRecord>> getRecords({int limit = 50, int offset = 0}) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/records?limit=$limit&offset=$offset'),
    );
    
    if (response.statusCode == 200) {
      final List<dynamic> data = jsonDecode(response.body);
      return data.map((e) => ClipboardRecord.fromJson(e)).toList();
    }
    
    throw Exception('Failed to load records: ${response.statusCode}');
  }

  Future<List<ClipboardRecord>> searchRecords(String query) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/search?q=${Uri.encodeComponent(query)}'),
    );

    if (response.statusCode == 200) {
      final List<dynamic> data = jsonDecode(response.body);
      return data.map((e) => ClipboardRecord.fromJson(e)).toList();
    }

    throw Exception('Failed to search: ${response.statusCode}');
  }

  Future<void> syncText(String text) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/sync'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'text': text}),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to sync text: ${response.statusCode}');
    }
  }

  Future<bool> toggleStar(String id) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/star/$id'),
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['starred'] ?? false;
    }

    throw Exception('Failed to toggle star');
  }

  Future<void> deleteRecord(String id) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/delete/$id'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to delete record');
    }
  }

  Future<void> pasteRecord(String id) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/paste/$id'),
    );

    if (response.statusCode != 200) {
      throw Exception('Failed to paste record');
    }
  }

  Future<String> getServerIp() async {
    final response = await http.get(Uri.parse('$baseUrl/api/ip'));
    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['ip'] ?? 'Unknown';
    }
    return 'Unknown';
  }

  Future<Map<String, dynamic>> getPairCode() async {
    final response = await http.get(Uri.parse('$baseUrl/api/pair-code'));
    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    }
    throw Exception('Failed to get pair code');
  }

  Future<bool> verifyPairCode(String code) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/verify'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'code': code}),
    );
    
    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['success'] ?? false;
    }
    return false;
  }
}

/// Simple TCP client for sending text to PC.
/// Uses HTTP by default, falls back to TCP if needed.
class TcpSyncClient {
  Socket? _socket;
  final Function(String) _onReceive;
  bool _connected = false;
  Timer? _reconnectTimer;
  String? _host;
  int? _port;
  
  TcpSyncClient(this._onReceive);
  
  Future<bool> connect(String host, int port) async {
    _host = host;
    _port = port;
    return _doConnect();
  }
  
  Future<bool> _doConnect() async {
    try {
      _socket?.close();
      _socket = await Socket.connect(_host!, _port!, timeout: Duration(seconds: 5));
      _connected = true;
      print('[TcpSyncClient] Connected to $_host:$_port');
      
      _socket!.listen(
        (data) {
          final message = String.fromCharCodes(data);
          print('[TcpSyncClient] Received: ${message.substring(0, message.length.clamp(0, 100))}');
          _onReceive(message);
        },
        onError: (e) {
          print('[TcpSyncClient] Error: $e');
          _connected = false;
          _scheduleReconnect();
        },
        onDone: () {
          print('[TcpSyncClient] Connection closed');
          _connected = false;
          _scheduleReconnect();
        },
        cancelOnError: true,
      );
      
      return true;
    } catch (e) {
      print('[TcpSyncClient] Connect failed: $e');
      _connected = false;
      _scheduleReconnect();
      return false;
    }
  }
  
  void _scheduleReconnect() {
    if (_reconnectTimer != null) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(Duration(seconds: 5), () {
      _reconnectTimer = null;
      if (_host != null) {
        print('[TcpSyncClient] Reconnecting...');
        _doConnect();
      }
    });
  }
  
  void send(String message) {
    if (_connected && _socket != null) {
      _socket!.write('$message\n');
      _socket!.flush();
    }
  }
  
  void close() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _host = null;
    _socket?.close();
    _connected = false;
  }
  
  bool get isConnected => _connected;
}
