import 'dart:io' show Directory, File; // desktop
import 'dart:typed_data';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';
import '../services/web_zip_read.dart';
import '../widgets/drop_area.dart';

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
  // web state
  Uint8List? _webBytes;
  String? _webName;
  List<String> _entries = [];
  final Set<String> _selected = {};

  Future<void> _prepareSample() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final inFile = File('${tmp.path}/a.txt')..writeAsStringSync('hello');
    final zip = '${tmp.path}/sample.zip';
    await _cli.create(zip, [inFile.path]);
    setState(() { _archive = zip; _outDir = '${tmp.path}/extracted'; Directory(_outDir!).createSync(recursive: true); });
  }

  Future<void> _pickArchive() async {
    final file = await openFile(acceptedTypeGroups: const [XTypeGroup(label: 'ZIP', extensions: ['zip'])]);
    if (file == null) return;
    if (kIsWeb) {
      _webBytes = await file.readAsBytes();
      _webName = file.name;
      try {
        _entries = WebZipReadService().list(_webBytes!);
        _selected
          ..clear()
          ..addAll(_entries);
        setState(() { _status = 'Ready'; });
      } catch (e) {
        setState(() { _status = 'Failed to read: $e'; });
      }
    } else {
      setState(() => _archive = file.path);
    }
  }

  Future<void> _pickOutDir() async {
    final dir = await getDirectoryPath();
    if (dir != null) setState(() => _outDir = dir);
  }

  Future<void> _runExtract() async {
    if (kIsWeb) {
      if (_webBytes == null) return;
      setState(() { _running = true; _status = 'Preparing downloads…'; });
      try {
        await WebZipReadService().extractSelected(_webBytes!, _selected.toList());
        setState(() { _running = false; _status = 'Downloaded'; });
      } catch (e) {
        setState(() { _running = false; _status = 'Failed: $e'; });
      }
      return;
    }
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
    return Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            OutlinedButton.icon(onPressed: _running ? null : _pickArchive, icon: const Icon(Icons.upload_file), label: const Text('Pick Archive')),
            const SizedBox(width: 8),
            if (!kIsWeb)
              OutlinedButton.icon(onPressed: _running ? null : _pickOutDir, icon: const Icon(Icons.folder_open), label: const Text('Output Folder')),
            const Spacer(),
            FilledButton.tonalIcon(
              onPressed: _running ? null : _runExtract,
              icon: const Icon(Icons.unarchive),
              label: Text(kIsWeb ? 'Download' : 'Extract'),
            ),
          ]),
          const SizedBox(height: 12),
          if (kIsWeb)
            Expanded(
              child: DecoratedBox(
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.grey.shade300),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: _webBytes == null
                    ? const Center(child: Text('Pick a ZIP to see contents'))
                    : Column(
                        children: [
                          Padding(
                            padding: const EdgeInsets.all(8.0),
                            child: Row(children: [
                              Text('Selected ${_selected.length}/${_entries.length}'),
                              const Spacer(),
                              TextButton(
                                onPressed: () => setState(() { _selected..clear()..addAll(_entries); }),
                                child: const Text('Select All'),
                              ),
                              TextButton(
                                onPressed: () => setState(() { _selected.clear(); }),
                                child: const Text('None'),
                              ),
                            ]),
                          ),
                          const Divider(height: 1),
                          Expanded(
                            child: ListView.builder(
                              itemCount: _entries.length,
                              itemBuilder: (_, i) {
                                final name = _entries[i];
                                final selected = _selected.contains(name);
                                return CheckboxListTile(
                                  dense: true,
                                  value: selected,
                                  onChanged: (v) {
                                    setState(() {
                                      if (v == true) { _selected.add(name); } else { _selected.remove(name); }
                                    });
                                  },
                                  title: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis),
                                );
                              },
                            ),
                          ),
                        ],
                      ),
              ),
            )
          else
            Expanded(
              child: DropArea(
                onDropped: (paths) {
                  final firstZip = paths.firstWhere(
                    (p) => p.toLowerCase().endsWith('.zip'),
                    orElse: () => '',
                  );
                  if (firstZip.isNotEmpty) setState(() => _archive = firstZip);
                },
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
          const SizedBox(height: 12),
          if (!kIsWeb) LinearProgressIndicator(value: _running ? null : (_pct == 0 ? null : _pct)),
          const SizedBox(height: 8),
          Text(_status),
          if (_error != null) Text(_error!, style: const TextStyle(color: Colors.red)),
        ]),
      );
  }
}
