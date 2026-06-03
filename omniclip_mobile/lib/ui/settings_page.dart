import 'package:flutter/material.dart';
import 'package:omniclip_mobile/utils/config.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({Key? key}) : super(key: key);

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  final TextEditingController _ipController = TextEditingController();
  final TextEditingController _portController = TextEditingController();
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadConfig();
  }

  Future<void> _loadConfig() async {
    setState(() {
      _ipController.text = Config.instance.serverIp;
      _portController.text = Config.instance.serverPort.toString();
      _isLoading = false;
    });
  }

  Future<void> _saveConfig() async {
    final ip = _ipController.text.trim();
    final port = int.tryParse(_portController.text) ?? 18911;
    
    if (ip.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入服务器 IP')),
      );
      return;
    }
    
    await Config.instance.updateBaseUrl(ip, port);
    
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('保存成功，重启应用生效')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('设置'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '服务器配置',
                      style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _ipController,
                      decoration: const InputDecoration(
                        labelText: '电脑 IP 地址',
                        hintText: '192.168.x.x',
                        border: OutlineInputBorder(),
                        prefixIcon: Icon(Icons.computer),
                      ),
                      keyboardType: TextInputType.numberWithOptions(decimal: true),
                    ),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _portController,
                      decoration: const InputDecoration(
                        labelText: '服务器端口',
                        hintText: '18911',
                        border: OutlineInputBorder(),
                        prefixIcon: Icon(Icons.settings_ethernet),
                      ),
                      keyboardType: TextInputType.number,
                    ),
                    const SizedBox(height: 24),
                    ElevatedButton(
                      onPressed: _saveConfig,
                      style: ElevatedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(vertical: 14),
                      ),
                      child: const Text('保存配置'),
                    ),
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
                  children: [
                    const Text(
                      '使用说明',
                      style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 12),
                    const Text(
                      '1. 在电脑端 OmniClip 同步页面查看本地 IP\n'
                      '2. 确保手机和电脑连接同一局域网（同一 WiFi）\n'
                      '3. 输入电脑 IP 和端口号（默认 18911）\n'
                      '4. 点击保存后重新加载记录\n'
                      '5. 如无法连接，请检查电脑防火墙设置',
                      style: TextStyle(fontSize: 14, height: 1.6),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
