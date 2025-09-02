import 'dart:io';
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';

class CompressScreen extends StatefulWidget {
  const CompressScreen({super.key});

  @override
  State<CompressScreen> createState() => _CompressScreenState();
}

class _CompressScreenState extends State<CompressScreen> {
  final _cli = RolyPolyCli();
  final List<String> _files = [];
  String? _archivePath;
  double _pct = 0;
  String _status = 'Idle';
  bool _running = false;

  Future<void> _addSampleFile() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final f = File('${tmp.path}/a.txt')..writeAsStringSync('hello');
    setState(() => _files.add(f.path));
  }

  Future<void> _runCreate() async {
    if (_files.isEmpty) return;
    setState(() { _running = true; _pct = 0; _status = 'Starting…'; });
    final tmp = Directory.systemTemp.createTempSync('rp');
    final zip = '${tmp.path}/out.zip';
    _archivePath = zip;
    await for (final evt in _cli.streamCreate(zip, _files)) {
      final event = evt['event'] as String?;
      if (event == 'progress') {
        final p = (evt['pct'] ?? 0) as num;
        setState(() { _pct = p.toDouble(); _status = 'Adding ${evt['file']}'; });
      } else if (event == 'start') {
        setState(() { _status = 'Creating…'; });
      } else if (event == 'done') {
        setState(() { _pct = 1; _status = 'Done'; _running = false; });
      }
    }
    setState(() { _running = false; });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly – Compress')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(children: [
              ElevatedButton(onPressed: _running ? null : _addSampleFile, child: const Text('Add Sample File')),
              const SizedBox(width: 12),
              ElevatedButton(onPressed: _running || _files.isEmpty ? null : _runCreate, child: const Text('Create Archive')),
            ]),
            const SizedBox(height: 16),
            Text('Files (${_files.length}):'),
            const SizedBox(height: 8),
            Expanded(
              child: Container(
                decoration: BoxDecoration(border: Border.all(color: Colors.grey.shade300), borderRadius: BorderRadius.circular(8)),
                child: ListView.builder(
                  itemCount: _files.length,
                  itemBuilder: (ctx, i) => ListTile(title: Text(_files[i], maxLines: 1, overflow: TextOverflow.ellipsis)),
                ),
              ),
            ),
            const SizedBox(height: 12),
            LinearProgressIndicator(value: _running ? null : (_pct == 0 ? null : _pct)),
            const SizedBox(height: 8),
            Text(_status),
            if (_archivePath != null) Text('Archive: $_archivePath'),
          ],
        ),
      ),
    );
  }
}

