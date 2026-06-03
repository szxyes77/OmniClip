import 'dart:async';
import 'package:flutter/material.dart';
import 'package:omniclip_mobile/api/client.dart';
import 'package:omniclip_mobile/utils/config.dart';
import 'package:omniclip_mobile/services/sse_client.dart';
import 'package:flutter_slidable/flutter_slidable.dart';

class RecordsPage extends StatefulWidget {
  const RecordsPage({Key? key}) : super(key: key);

  @override
  State<RecordsPage> createState() => _RecordsPageState();
}

class _RecordsPageState extends State<RecordsPage> {
  List<ClipboardRecord> _records = [];
  bool _isLoading = true;
  bool _isSearching = false;
  String _searchQuery = '';
  final TextEditingController _searchController = TextEditingController();
  Timer? _refreshTimer;
  SSEClient? _sseClient;
  StreamSubscription? _sseSubscription;

  @override
  void initState() {
    super.initState();
    _loadRecords();
    // Start SSE connection for real-time updates
    _startSSE();
    // Fallback: 每5秒自动刷新记录 (SSE may not always be available)
    _refreshTimer = Timer.periodic(Duration(seconds: 5), (_) {
      if (mounted && !_isSearching) {
        _loadRecords(silent: true);
      }
    });
  }

  void _startSSE() {
    _sseClient = SSEClient(Config.instance.baseUrl);
    _sseSubscription = _sseClient!.listen().listen((event) {
      if (!mounted) return;
      
      if (event['type'] == 'clipboard-update') {
        print('[SSE] Received clipboard-update event');
        // Immediately refresh records when PC clipboard changes
        _loadRecords(silent: true);
        
        // Show notification
        if (event['data'] != null && event['data'] is Map) {
          final content = event['data']['content'] ?? '';
          if (content.toString().isNotEmpty) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text('📋 收到新剪贴板: ${content.toString().substring(0, content.toString().length > 30 ? 30 : null)}...'),
                duration: Duration(seconds: 2),
                behavior: SnackBarBehavior.floating,
              ),
            );
          }
        }
      }
    });
  }

  @override
  void dispose() {
    _sseSubscription?.cancel();
    _sseClient?.dispose();
    _refreshTimer?.cancel();
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _loadRecords({bool silent = false}) async {
    if (_searchQuery.isNotEmpty) {
      await _searchRecords();
      return;
    }
    
    if (!silent) {
      setState(() {
        _isLoading = true;
      });
    }
    
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      final records = await api.getRecords(limit: 100);
      if (mounted) {
        setState(() {
          _records = records;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted && !silent) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _searchRecords() async {
    if (!mounted) return;
    
    setState(() {
      _isLoading = true;
      _isSearching = true;
    });
    
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      final records = await api.searchRecords(_searchQuery);
      if (mounted) {
        setState(() {
          _records = records;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('搜索失败: $e')),
        );
      }
    }
  }

  Future<void> _toggleStar(String id) async {
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      final starred = await api.toggleStar(id);
      final record = _records.firstWhere((r) => r.id == id);
      
      setState(() {
        final idx = _records.indexWhere((r) => r.id == id);
        if (idx != -1) {
          _records[idx] = ClipboardRecord(
            id: record.id,
            recordType: record.recordType,
            content: record.content,
            title: record.title,
            contentHash: record.contentHash,
            createdAt: record.createdAt,
            updatedAt: record.updatedAt,
            lastUsedAt: record.lastUsedAt,
            copyCount: record.copyCount,
            starCount: record.starCount,
            isStarred: starred,
            thumbnailPath: record.thumbnailPath,
            tags: record.tags,
            sourceApp: record.sourceApp,
          );
        }
      });
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('操作失败: $e')),
      );
    }
  }

  Future<void> _deleteRecord(String id) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('确认删除'),
        content: Text('确定要删除此记录吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text('删除', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
    
    if (confirmed != true) return;
    
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      await api.deleteRecord(id);
      
      // Check if record exists before removing
      setState(() {
        final idx = _records.indexWhere((r) => r.id == id);
        if (idx != -1) {
          _records.removeAt(idx);
        }
      });
      
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('已删除')),
      );
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('删除失败: $e')),
      );
    }
  }

  Future<void> _copyToClipboard(String id, String content) async {
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      await api.pasteRecord(id);
      
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('已复制到电脑剪贴板')),
      );
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('复制失败: $e')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading && _records.isEmpty) {
      return Center(child: CircularProgressIndicator());
    }
    
    if (_records.isEmpty && !_isSearching) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.content_paste_outlined, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text(
              '暂无剪贴板记录',
              style: TextStyle(fontSize: 16, color: Colors.grey),
            ),
            SizedBox(height: 8),
            Text(
              '在电脑复制后这里会自动显示',
              style: TextStyle(fontSize: 14, color: Colors.grey.shade400),
            ),
          ],
        ),
      );
    }

    return Column(
      children: [
        Padding(
          padding: EdgeInsets.all(12),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _searchController,
                  decoration: InputDecoration(
                    hintText: '搜索剪贴板记录...',
                    prefixIcon: Icon(Icons.search),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    contentPadding: EdgeInsets.symmetric(vertical: 0),
                  ),
                  onSubmitted: (value) {
                    _searchQuery = value.trim();
                    if (_searchQuery.isEmpty) {
                      _isSearching = false;
                    }
                    _loadRecords();
                  },
                ),
              ),
              SizedBox(width: 8),
              IconButton(
                icon: Icon(Icons.refresh),
                onPressed: _loadRecords,
                tooltip: '刷新',
              ),
            ],
          ),
        ),
        Expanded(
          child: ListView.builder(
            itemCount: _records.length,
            padding: EdgeInsets.only(bottom: 16),
            itemBuilder: (context, index) {
              final record = _records[index];
              return _buildRecordCard(record);
            },
          ),
        ),
      ],
    );
  }

  Widget _buildRecordCard(ClipboardRecord record) {
    final isText = record.recordType == 'text';
    
    return Container(
      margin: EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: Slidable(
        key: ValueKey(record.id),
        endActionPane: ActionPane(
          motion: ScrollMotion(),
          children: [
            SlidableAction(
              onPressed: (_) => _copyToClipboard(record.id, record.content),
              backgroundColor: Color(0xFF1976D2),
              foregroundColor: Colors.white,
              icon: Icons.copy,
              label: '复制',
            ),
            SlidableAction(
              onPressed: (_) => _toggleStar(record.id),
              backgroundColor: record.isStarred ? Colors.orange : Colors.grey,
              foregroundColor: Colors.white,
              icon: record.isStarred ? Icons.star : Icons.star_border,
              label: record.isStarred ? '取消收藏' : '收藏',
            ),
            SlidableAction(
              onPressed: (_) => _deleteRecord(record.id),
              backgroundColor: Colors.red,
              foregroundColor: Colors.white,
              icon: Icons.delete,
              label: '删除',
            ),
          ],
        ),
        child: Card(
          elevation: 2,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          child: InkWell(
            onTap: () => _copyToClipboard(record.id, record.content),
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (!isText)
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: Color(0xFFE3F2FD),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        record.recordType.toUpperCase(),
                        style: TextStyle(
                          fontSize: 11,
                          color: Color(0xFF1976D2),
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                  SizedBox(height: 8),
                  Text(
                    record.content,
                    maxLines: 4,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 15,
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
                      SizedBox(width: 12),
                      Icon(Icons.copy, size: 14, color: Colors.grey),
                      SizedBox(width: 4),
                      Text(
                        '${record.copyCount} 次',
                        style: TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                      Spacer(),
                      if (record.isStarred)
                        Icon(Icons.star, size: 16, color: Colors.orange),
                    ],
                  ),
                ],
              ),
            ),
          ),
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
      if (diff.inDays < 7) return '${diff.inDays}天前';
      
      return '${time.month}/${time.day} ${time.hour.toString().padLeft(2, '0')}:${time.minute.toString().padLeft(2, '0')}';
    } catch (e) {
      return timeStr;
    }
  }
}
