import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:omniclip_mobile/api/client.dart';
import 'package:omniclip_mobile/utils/config.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:http/http.dart' as http;
import 'dart:convert';

class SendPage extends StatefulWidget {
  const SendPage({Key? key}) : super(key: key);

  @override
  State<SendPage> createState() => _SendPageState();
}

class _SendPageState extends State<SendPage> {
  List<ClipboardRecord> _records = [];
  bool _isLoading = true;
  bool _isSending = false;
  final TextEditingController _ipController = TextEditingController();
  final TextEditingController _portController = TextEditingController(text: '18911');

  @override
  void initState() {
    super.initState();
    _loadClipboardData();
    _ipController.text = Config.instance.serverIp;
  }

  @override
  void dispose() {
    _ipController.dispose();
    _portController.dispose();
    super.dispose();
  }

  Future<void> _loadClipboardData() async {
    setState(() {
      _isLoading = true;
    });
    
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      final records = await api.getRecords(limit: 50);
      setState(() {
        _records = records;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('获取剪贴板数据失败: $e')),
      );
    }
  }

  /// Send text to PC via HTTP to the user-specified IP:port
  Future<void> _sendTextToDevice(String content) async {
    final ip = _ipController.text.trim();
    if (ip.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('请输入对方 IP')),
      );
      return;
    }
    
    final port = int.tryParse(_portController.text) ?? 18911;

    setState(() => _isSending = true);
    
    try {
      // Send via HTTP POST to PC's HTTP server (port 18911 by default)
      final url = 'http://$ip:$port/api/sync';
      final response = await http.post(
        Uri.parse(url),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode({'text': content}),
      ).timeout(Duration(seconds: 10));
      
      if (response.statusCode == 200) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('已发送到 $ip:$port')),
        );
        _loadClipboardData();
      } else {
        throw Exception('HTTP ${response.statusCode}');
      }
    } catch (e) {
      String errorMsg = '发送失败';
      if (e.toString().contains('Connection refused') || e.toString().contains('Failed host lookup')) {
        errorMsg = '无法连接到 $ip:$port，请检查 IP 和端口是否正确';
      } else if (e.toString().contains('Timeout')) {
        errorMsg = '连接超时，请检查网络';
      } else {
        errorMsg = '发送失败: $e';
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(errorMsg), duration: Duration(seconds: 3)),
      );
    } finally {
      setState(() => _isSending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Center(child: CircularProgressIndicator());
    }
    
    return SingleChildScrollView(
      padding: EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '目标设备',
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  SizedBox(height: 12),
                  TextField(
                    controller: _ipController,
                    decoration: InputDecoration(
                      labelText: '对方 IP',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                      prefixIcon: Icon(Icons.computer),
                    ),
                  ),
                  SizedBox(height: 12),
                  TextField(
                    controller: _portController,
                    decoration: InputDecoration(
                      labelText: '端口 (默认 18911)',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                      prefixIcon: Icon(Icons.portable_wifi_off),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ],
              ),
            ),
          ),
          SizedBox(height: 20),
          Text(
            '当前剪贴板内容',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
          ),
          SizedBox(height: 8),
          if (_records.isEmpty)
            Center(
              child: Padding(
                padding: EdgeInsets.all(40),
                child: Column(
                  children: [
                    Icon(Icons.content_paste_off, size: 48, color: Colors.grey),
                    SizedBox(height: 16),
                    Text(
                      '暂无剪贴板内容',
                      style: TextStyle(color: Colors.grey),
                    ),
                  ],
                ),
              ),
            )
          else
            ..._records.map((record) => _buildRecordCard(record)).toList(),
        ],
      ),
    );
  }

  Widget _buildRecordCard(ClipboardRecord record) {
    return Card(
      margin: EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: record.recordType == 'text' 
                    ? Color(0xFFE8F5E9) 
                    : Color(0xFFE3F2FD),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                record.recordType.toUpperCase(),
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: record.recordType == 'text' 
                      ? Color(0xFF2E7D32) 
                      : Color(0xFF1976D2),
                ),
              ),
            ),
            SizedBox(height: 8),
            Text(
              record.content,
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: 14,
                height: 1.4,
              ),
            ),
            SizedBox(height: 12),
            Row(
              children: [
                Icon(Icons.access_time, size: 14, color: Colors.grey),
                SizedBox(width: 4),
                Text(
                  _formatTime(record.updatedAt),
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
                Spacer(),
                Text(
                  '已使用 ${record.copyCount} 次',
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
            SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton.icon(
                onPressed: _isSending ? null : () => _sendTextToDevice(record.content),
                icon: _isSending 
                    ? SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                    : Icon(Icons.send),
                label: Text('发送到 ${_ipController.text}'),
                style: ElevatedButton.styleFrom(
                  padding: EdgeInsets.symmetric(vertical: 12),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(String timeStr) {
    try {
      final time = DateTime.parse(timeStr);
      final now = DateTime.now();
      final diff = now.difference(time);
      
      if (diff.inMinutes < 1) return '刚刚';
      if (diff.inMinutes < 60) return '${diff.inMinutes}分钟前';
      if (diff.inHours < 24) return '${diff.inHours}小时前';
      
      return '${time.month}/${time.day} ${time.hour}:${time.minute.toString().padLeft(2, '0')}';
    } catch (e) {
      return timeStr;
    }
  }
}
