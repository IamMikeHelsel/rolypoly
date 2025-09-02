import 'package:flex_color_scheme/flex_color_scheme.dart';
import 'package:flutter/material.dart';
import 'features/compress.dart';
import 'features/extract.dart';
import 'features/inspect.dart';
import 'features/validate_stats.dart';

void main() {
  runApp(const RolyPolyApp());
}

class RolyPolyApp extends StatelessWidget {
  const RolyPolyApp({super.key});

  @override
  Widget build(BuildContext context) {
    final light = FlexThemeData.light(scheme: FlexScheme.damask);
    final dark = FlexThemeData.dark(scheme: FlexScheme.ebonyClay);
    return MaterialApp(
      title: 'RolyPoly',
      theme: light,
      darkTheme: dark,
      home: const _Home(),
      debugShowCheckedModeBanner: false,
    );
  }
}

class _Home extends StatefulWidget { const _Home(); @override State<_Home> createState() => _HomeState(); }
class _HomeState extends State<_Home> {
  int _index = 0;
  final _pages = const [CompressScreen(), ExtractScreen(), InspectScreen(), ValidateStatsScreen()];
  final _titles = const ['Compress', 'Extract', 'Inspect', 'Validate & Stats'];
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('RolyPoly – ${_titles[_index]}')),
      body: _pages[_index],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        destinations: const [
          NavigationDestination(icon: Icon(Icons.archive), label: 'Compress'),
          NavigationDestination(icon: Icon(Icons.unarchive), label: 'Extract'),
          NavigationDestination(icon: Icon(Icons.list), label: 'Inspect'),
          NavigationDestination(icon: Icon(Icons.verified), label: 'Validate'),
        ],
        onDestinationSelected: (i) => setState(() => _index = i),
      ),
    );
  }
}
