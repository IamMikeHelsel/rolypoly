import 'package:flex_color_scheme/flex_color_scheme.dart';
import 'package:flutter/material.dart';
import 'features/compress.dart';

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
      home: const CompressScreen(),
      debugShowCheckedModeBanner: false,
    );
  }
}

class _Home extends StatelessWidget {
  const _Home();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly')),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: const [
            Text('Welcome to RolyPoly'),
            SizedBox(height: 8),
            Text('CLI-first. Flutter GUI coming soon.'),
          ],
        ),
      ),
    );
  }
}
