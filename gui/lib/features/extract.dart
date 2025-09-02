import 'dart:io';
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';

class ExtractScreen extends StatefulWidget {
  const ExtractScreen({super.key});
  @override
  State<ExtractScreen> createState() => _ExtractScreenState();
}

class _ExtractScreenState extends State<ExtractScreen> {
  final _cli = RolyPolyCli();
  String? _archive;
  String? _outDir;
  double _pct = 0;
  String _status = 'Idle';
  bool _running = false;

  Future<void> _prepareSample() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final inFile = File('${tmp.path}/a.txt')..writeAsStringSync('hello');
    final zip = '${tmp.path}/sample.zip';
    await _cli.create(zip, [inFile.path]);
    setState(() { _archive = zip; _outDir = '${tmp.path}/extracted'; Directory(_outDir!).createSync(recursive: true); });
  }

  Future<void> _runExtract() async {
    if (_archive == null || _outDir == null) return;
    setState(() { _running = true; _pct = 0; _status = 'Starting…'; });
    await for (final evt in _cli.streamExtract(_archive!, _outDir!)) {
      final event = evt['event'] as String?;
      if (event == 'progress') {
        setState(() { _pct = (evt['pct'] ?? 0).toDouble(); _status = 'Extracting ${evt['file']}'; });
      } else if (event == 'start') {
        setState(() { _status = 'Extracting…'; });
      } else if (event == 'done') {
        setState(() { _pct = 1; _status = 'Done'; _running = false; });
      }
    }
    setState(() { _running = false; });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly – Extract')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            ElevatedButton(onPressed: _running ? null : _prepareSample, child: const Text('Prepare Sample')),
            const SizedBox(width: 12),
            ElevatedButton(onPressed: _running || _archive == null ? null : _runExtract, child: const Text('Extract')),
          ]),
          const SizedBox(height: 12),
          Text('Archive: ${_archive ?? '-'}'),
          Text('Output: ${_outDir ?? '-'}'),
          const SizedBox(height: 12),
          LinearProgressIndicator(value: _running ? null : (_pct == 0 ? null : _pct)),
          const SizedBox(height: 8),
          Text(_status),
        ]),
      ),
    );
  }
}

