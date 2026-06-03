import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;

/// Lightweight SSE client for receiving real-time events from PC
class SSEClient {
  final String baseUrl;
  http.Client? _client;
  StreamController<Map<String, dynamic>>? _eventController;
  Timer? _reconnectTimer;
  bool _disposed = false;

  SSEClient(this.baseUrl);

  /// Start listening to SSE events. Returns a stream of events.
  Stream<Map<String, dynamic>> listen() {
    _eventController = StreamController<Map<String, dynamic>>.broadcast();
    _connect();
    return _eventController!.stream;
  }

  Future<void> _connect() async {
    if (_disposed) return;
    
    _client = http.Client();
    final url = '$baseUrl/api/events';
    
    try {
      final response = await _client!.get(
        Uri.parse(url),
        headers: {'Accept': 'text/event-stream'},
      ).timeout(Duration(seconds: 15));
      
      if (response.statusCode != 200) {
        _scheduleReconnect();
        return;
      }
      
      // Read SSE stream body
      final stream = response.stream;
      String buffer = '';
      
      await for (final chunk in stream) {
        if (_disposed) return;
        
        buffer += String.fromCharCodes(chunk);
        
        // Process complete SSE events (separated by \n\n)
        while (buffer.contains('\n\n')) {
          final idx = buffer.indexOf('\n\n');
          final eventStr = buffer.substring(0, idx);
          buffer = buffer.substring(idx + 2);
          
          final event = _parseSSEEvent(eventStr);
          if (event != null) {
            _eventController?.add(event);
          }
        }
      }
    } catch (e) {
      print('[SSE] Connection error: $e');
    }
    
    if (!_disposed) {
      _scheduleReconnect();
    }
  }

  Map<String, dynamic>? _parseSSEEvent(String eventStr) {
    String eventType = 'message';
    String? data;
    
    for (final line in eventStr.split('\n')) {
      if (line.startsWith('event:')) {
        eventType = line.substring(6).trim();
      } else if (line.startsWith('data:')) {
        data = line.substring(5).trim();
      }
    }
    
    if (data != null) {
      try {
        return {
          'type': eventType,
          'data': jsonDecode(data),
        };
      } catch (e) {
        return {'type': eventType, 'data': data};
      }
    }
    return null;
  }

  void _scheduleReconnect() {
    if (_disposed) return;
    
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(Duration(seconds: 3), () {
      if (!_disposed) {
        print('[SSE] Reconnecting...');
        _connect();
      }
    });
  }

  void dispose() {
    _disposed = true;
    _reconnectTimer?.cancel();
    _eventController?.close();
    _client?.close();
  }
}
