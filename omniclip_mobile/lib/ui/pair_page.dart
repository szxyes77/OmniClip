import 'dart:math';
import 'package:flutter/material.dart';
import 'package:omniclip_mobile/api/client.dart';
import 'package:omniclip_mobile/utils/config.dart';
import 'package:shared_preferences/shared_preferences.dart';

class PairPage extends StatefulWidget {
  const PairPage({Key? key}) : super(key: key);

  @override
  State<PairPage> createState() => _PairPageState();
}

class _PairPageState extends State<PairPage> {
  String _myPairCode = '------';
  String? _pcPairCode;
  bool _isLoading = true;
  bool _isPaired = false;
  final TextEditingController _codeController = TextEditingController();
  String _statusMsg = '';

  @override
  void initState() {
    super.initState();
    _initPairing();
  }

  Future<void> _initPairing() async {
    setState(() => _isLoading = true);
    try {
      // Load or generate my pair code
      final prefs = await SharedPreferences.getInstance();
      var code = prefs.getString('my_pair_code');
      if (code == null || code.length != 6) {
        final rnd = Random();
        code = (100000 + rnd.nextInt(900000)).toString();
        await prefs.setString('my_pair_code', code);
      }
      
      // Fetch PC's pair code
      final api = OmniClipApi(Config.instance.baseUrl);
      final pcCodeData = await api.getPairCode();
      final pcCode = pcCodeData['code']?.toString();
      
      // Check if already paired
      final isPaired = prefs.getBool('is_paired') ?? false;
      
      setState(() {
        _myPairCode = code;
        _pcPairCode = pcCode;
        _isPaired = isPaired;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('获取配对码失败: $e')),
        );
      }
    }
  }

  Future<void> _verifyCode(String code) async {
    if (code.length != 6) {
      setState(() => _statusMsg = '请输入 6 位配对码');
      return;
    }

    setState(() => _statusMsg = '正在验证...');
    try {
      final api = OmniClipApi(Config.instance.baseUrl);
      final success = await api.verifyPairCode(code);
      if (success) {
        final prefs = await SharedPreferences.getInstance();
        await prefs.setBool('is_paired', true);
        setState(() {
          _isPaired = true;
          _statusMsg = '配对成功！';
        });
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('配对成功！已连接到电脑')),
          );
        }
      } else {
        setState(() => _statusMsg = '配对码无效，请重试');
      }
    } catch (e) {
      setState(() => _statusMsg = '验证失败: $e');
    }
    Future.delayed(const Duration(seconds: 3), () {
      if (mounted) setState(() => _statusMsg = '');
    });
  }

  Future<void> _refreshMyCode() async {
    final prefs = await SharedPreferences.getInstance();
    final rnd = Random();
    final code = (100000 + rnd.nextInt(900000)).toString();
    await prefs.setString('my_pair_code', code);
    await prefs.setBool('is_paired', false);
    setState(() {
      _myPairCode = code;
      _isPaired = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('配对连接')),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: _initPairing,
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    // My pair code - for PC to read
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Column(
                          children: [
                            const Icon(Icons.phone_android, size: 48, color: Colors.blue),
                            const SizedBox(height: 8),
                            const Text('我的配对码', style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
                            const SizedBox(height: 4),
                            Text('电脑端输入此码完成配对', style: Theme.of(context).textTheme.bodySmall),
                            const SizedBox(height: 12),
                            Container(
                              padding: const EdgeInsets.all(16),
                              decoration: BoxDecoration(
                                color: Colors.blue.shade50,
                                borderRadius: BorderRadius.circular(12),
                                border: Border.all(color: Colors.blue.shade200),
                              ),
                              child: Text(
                                _myPairCode,
                                style: const TextStyle(fontSize: 36, fontWeight: FontWeight.bold, letterSpacing: 8, color: Colors.blue),
                                textAlign: TextAlign.center,
                              ),
                            ),
                            const SizedBox(height: 8),
                            TextButton.icon(
                              onPressed: _refreshMyCode,
                              icon: const Icon(Icons.refresh, size: 18),
                              label: const Text('刷新配对码'),
                            ),
                          ],
                        ),
                      ),
                    ),
                    const SizedBox(height: 16),
                    // PC's pair code - for reference
                    if (_pcPairCode != null)
                      Card(
                        color: Colors.grey.shade50,
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: Column(
                            children: [
                              const Icon(Icons.computer, size: 36, color: Colors.grey),
                              const SizedBox(height: 8),
                              const Text('电脑配对码', style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold)),
                              const SizedBox(height: 8),
                              Text(
                                _pcPairCode!,
                                style: const TextStyle(fontSize: 28, fontWeight: FontWeight.bold, letterSpacing: 6, color: Colors.grey),
                                textAlign: TextAlign.center,
                              ),
                            ],
                          ),
                        ),
                      ),
                    const SizedBox(height: 16),
                    // Verification result or input
                    if (_isPaired)
                      Card(
                        color: Colors.green.shade50,
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: Column(
                            children: const [
                              Icon(Icons.check_circle, size: 64, color: Colors.green),
                              SizedBox(height: 16),
                              Text('已配对成功！', style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold)),
                              SizedBox(height: 8),
                              Text('你现在可以同步剪贴板内容了', textAlign: TextAlign.center),
                            ],
                          ),
                        ),
                      )
                    else
                      Card(
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              const Text('输入电脑配对码', style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
                              const SizedBox(height: 12),
                              TextField(
                                controller: _codeController,
                                keyboardType: TextInputType.number,
                                maxLength: 6,
                                textAlign: TextAlign.center,
                                style: const TextStyle(fontSize: 32, letterSpacing: 8),
                                decoration: const InputDecoration(counterText: '', border: OutlineInputBorder(), hintText: '000000'),
                              ),
                              const SizedBox(height: 12),
                              ElevatedButton.icon(
                                onPressed: () => _verifyCode(_codeController.text),
                                icon: const Icon(Icons.check),
                                label: const Text('验证配对码'),
                                style: ElevatedButton.styleFrom(padding: const EdgeInsets.symmetric(vertical: 12)),
                              ),
                              if (_statusMsg.isNotEmpty) ...[
                                const SizedBox(height: 8),
                                Text(_statusMsg, style: TextStyle(color: _statusMsg.contains('成功') ? Colors.green : Colors.red, fontSize: 13)),
                              ],
                            ],
                          ),
                        ),
                      ),
                    const SizedBox(height: 16),
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: const [
                            Text('如何配对？', style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
                            SizedBox(height: 8),
                            Text('1. 确保手机和电脑连接同一局域网\n2. 在"设置"页面配置电脑 IP 地址\n3. 将手机配对码输入到电脑端 OmniClip\n4. 将电脑配对码输入到此处\n5. 配对成功后即可双向同步剪贴板'),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
    );
  }
}
