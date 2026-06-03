import 'package:flutter/material.dart';
import 'package:omniclip_mobile/ui/records_page.dart';
import 'package:omniclip_mobile/ui/pair_page.dart';
import 'package:omniclip_mobile/ui/send_page.dart';
import 'package:omniclip_mobile/ui/settings_page.dart';

class MainScreen extends StatefulWidget {
  const MainScreen({Key? key}) : super(key: key);

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  int _currentIndex = 0;

  final List<Widget> _pages = [
    const RecordsPage(),
    const PairPage(),
    const SendPage(),
    const SettingsPage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _pages,
      ),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: _currentIndex,
        onTap: (index) {
          setState(() {
            _currentIndex = index;
          });
        },
        type: BottomNavigationBarType.fixed,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.content_paste),
            label: '记录',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.link),
            label: '配对',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.send),
            label: '发送',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings),
            label: '设置',
          ),
        ],
      ),
    );
  }
}
