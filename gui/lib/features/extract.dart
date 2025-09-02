import 'dart:io';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_selector/file_selector.dart';
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
  bool _dragging = false;
  String? _error;

  Future<void> _prepareSample() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final inFile = File('${tmp.path}/a.txt')..writeAsStringSync('hello');
    final zip = '${tmp.path}/sample.zip';
    await _cli.create(zip, [inFile.path]);
    setState(() { _archive = zip; _outDir = '${tmp.path}/extracted'; Directory(_outDir!).createSync(recursive: true); });
  }

  Future<void> _pickArchive() async {
    final file = await openFile(acceptedTypeGroups: const [XTypeGroup(label: 'ZIP', extensions: ['zip'])]);
    if (file != null) setState(() => _archive = file.path);
  }

  Future<void> _pickOutDir() async {
    final dir = await getDirectoryPath();
    if (dir != null) setState(() => _outDir = dir);
  }

  Future<void> _runExtract() async {
    if (_archive == null || _outDir == null) return;
    setState(() { _running = true; _pct = 0; _status = 'Starting…'; _error = null; });
    try {
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
    } catch (e) {
      setState(() { _error = e.toString(); _running = false; });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly – Extract')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            OutlinedButton.icon(onPressed: _running ? null : _pickArchive, icon: const Icon(Icons.upload_file), label: const Text('Pick Archive')),
            const SizedBox(width: 8),
            OutlinedButton.icon(onPressed: _running ? null : _pickOutDir, icon: const Icon(Icons.folder_open), label: const Text('Output Folder')),
            const Spacer(),
            FilledButton.icon(onPressed: _running || _archive == null || _outDir == null ? null : _runExtract, icon: const Icon(Icons.unarchive), label: const Text('Extract')),
          ]),
          const SizedBox(height: 12),
          Expanded(
            child: DropTarget(
              onDragEntered: (_) => setState(() => _dragging = true),
              onDragExited: (_) => setState(() => _dragging = false),
              onDragDone: (d) {
                final firstZip = d.files.map((f) => f.path).whereType<String>().firstWhere(
                      (p) => p.toLowerCase().endsWith('.zip'),
                      orElse: () => '',
                    );
                if (firstZip.isNotEmpty) setState(() => _archive = firstZip);
                setState(() => _dragging = false);
              },
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: _dragging ? Colors.blue.withOpacity(0.06) : Colors.transparent,
                  border: Border.all(color: _dragging ? Colors.blue : Colors.grey.shade300),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.file_upload, size: 48, color: Colors.grey),
                        const SizedBox(height: 8),
                        Text('Archive: ${_archive ?? '-'}'),
                        const SizedBox(height: 8),
                        Text('Output dir: ${_outDir ?? '-'}'),
                        const SizedBox(height: 8),
                        const Text('Drag & drop a .zip here or use Pick Archive'),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 12),
          LinearProgressIndicator(value: _running ? null : (_pct == 0 ? null : _pct)),
          const SizedBox(height: 8),
          Text(_status),
          if (_error != null) Text(_error!, style: const TextStyle(color: Colors.red)),
        ]),
      ),
    );
  }
}
