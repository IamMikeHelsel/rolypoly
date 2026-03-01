import 'package:flex_color_scheme/flex_color_scheme.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';
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
    final light = FlexThemeData.light(
      scheme: FlexScheme.indigo,
      useMaterial3: true,
      appBarElevation: 0,
    );
    final dark = FlexThemeData.dark(
      scheme: FlexScheme.indigo,
      useMaterial3: true,
      appBarElevation: 0,
    );
    return MaterialApp(
      title: 'RolyPoly',
      theme: light,
      darkTheme: dark,
      themeMode: ThemeMode.dark,
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
    const appVersion = String.fromEnvironment('APP_VERSION', defaultValue: 'dev');
    final banner = kIsWeb
        ? MaterialBanner(
            content: const Text('Web preview: operations require a backend or will be limited. Use desktop app for full functionality.'),
            actions: [
              TextButton(onPressed: () => ScaffoldMessenger.of(context).hideCurrentMaterialBanner(), child: const Text('Dismiss'))
            ],
          )
        : null;

    return Scaffold(
      appBar: AppBar(title: Text('RolyPoly – ${_titles[_index]}')),
      body: Column(
        children: [
          if (banner != null) banner,
          Expanded(child: _pages[_index]),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: Row(
              children: [
                Text('v$appVersion', style: Theme.of(context).textTheme.bodySmall),
                const Spacer(),
                TextButton.icon(
                  onPressed: () => _showAbout(context, appVersion),
                  icon: const Icon(Icons.info_outline, size: 16),
                  label: const Text('About'),
                ),
                TextButton.icon(
                  onPressed: () => _openGitHub(context),
                  icon: const Icon(Icons.open_in_new, size: 16),
                  label: const Text('GitHub'),
                ),
              ],
            ),
          ),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        labelBehavior: NavigationDestinationLabelBehavior.alwaysHide,
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

Future<void> _openGitHub(BuildContext context) async {
  const url = 'https://github.com/user/rolypoly';
  final ok = await launchUrlString(url);
  if (!ok) {
    ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Repo: https://github.com/user/rolypoly')));
  }
}

void _showAbout(BuildContext context, String version) {
  showAboutDialog(
    context: context,
    applicationName: 'RolyPoly',
    applicationVersion: version,
    children: const [
      Text('Modern ZIP archiver — CLI, Desktop, and PWA.'),
      SizedBox(height: 8),
      Text('Desktop uses the Rust CLI for full performance. Web provides a convenient preview (client-side).'),
    ],
  );
}
