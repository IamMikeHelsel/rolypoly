import 'dart:io' show Directory, File;
import 'dart:typed_data';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';
import '../services/web_zip_service.dart';
import '../services/web_download.dart';
import '../widgets/drop_area.dart';
import '../services/fs_save.dart';

class CompressScreen extends StatefulWidget {
  const CompressScreen({super.key});

  @override
  State<CompressScreen> createState() => _CompressScreenState();
}

class _CompressScreenState extends State<CompressScreen> {
  final _cli = RolyPolyCli();
  final List<String> _inputs = [];
  final Map<String, Uint8List> _inputsWeb = {}; // web: name -> bytes
  String? _archivePath;
  double _pct = 0;
  String _status = 'Idle';
  bool _running = false;
  bool _dragging = false;
  String? _error;

  void _addPaths(Iterable<String> paths) {
    final next = [..._inputs];
    for (final p in paths) {
      if (!next.contains(p)) next.add(p);
    }
    setState(() => _inputs
      ..clear()
      ..addAll(next));
  }

  Future<void> _pickFiles() async {
    final files = await openFiles(acceptedTypeGroups: const [XTypeGroup(label: 'Any')]);
    if (files.isEmpty) return;
    if (kIsWeb) {
      for (final xf in files) {
        final bytes = await xf.readAsBytes();
        var name = xf.name;
        var i = 1;
        while (_inputsWeb.containsKey(name)) { name = "${xf.name}(${i++})"; }
        _inputsWeb[name] = bytes;
      }
      setState(() {});
    } else {
      _addPaths(files.map((e) => e.path));
    }
  }

  Future<void> _pickFolder() async {
    final dir = await getDirectoryPath();
    if (dir != null) _addPaths([dir]);
  }

  Future<void> _chooseOutput() async {
    final savePath = await pickSaveZip(suggestedName: 'archive.zip');
    if (savePath != null) setState(() => _archivePath = savePath);
  }

  Future<void> _runCreate() async {
    if ((kIsWeb && _inputsWeb.isEmpty) || (!kIsWeb && _inputs.isEmpty)) return;
    if (_running) return;
    setState(() {
      _running = true;
      _pct = 0;
      _status = 'Starting…';
      _error = null;
    });
    if (kIsWeb) {
      try {
        _status = 'Zipping in browser…';
        final data = await WebZipService().createZip(_inputsWeb);
        downloadBytes(data, 'archive.zip');
        setState(() { _pct = 1; _status = 'Downloaded archive.zip'; _running = false; });
      } catch (e) {
        setState(() { _error = e.toString(); _running = false; });
      }
      return;
    }

    var out = _archivePath;
    if (out == null || out.isEmpty) {
      final picked = await pickSaveZip(suggestedName: 'archive.zip');
      if (picked == null) { setState(() => _running = false); return; }
      out = picked;
      setState(() => _archivePath = out);
    }

    try {
      await for (final evt in _cli.streamCreate(out!, _inputs)) {
        final event = evt['event'] as String?;
        if (event == 'progress') {
          final total = (evt['total'] ?? 0) as num;
          final current = (evt['current'] ?? 0) as num;
          final rawPct = evt['pct'];
          final p = rawPct is num
              ? rawPct
              : (total > 0 ? (current / total) : 0);
          setState(() {
            _pct = p.toDouble().clamp(0.0, 1.0);
            _status = 'Adding ${evt['file'] ?? ''}';
          });
        } else if (event == 'start') {
          setState(() => _status = 'Creating…');
        } else if (event == 'done') {
          setState(() {
            _pct = 1;
            _status = 'Done';
            _running = false;
          });
        }
      }
    } catch (e) {
      setState(() {
        _error = e.toString();
        _running = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            FilledButton.icon(onPressed: _running ? null : _pickFiles, icon: const Icon(Icons.add), label: const Text('Add Files')),
            if (!kIsWeb) ...[
              const SizedBox(width: 8),
              OutlinedButton.icon(onPressed: _running ? null : _pickFolder, icon: const Icon(Icons.create_new_folder), label: const Text('Add Folder')),
            ],
            const Spacer(),
            if (!kIsWeb)
              OutlinedButton.icon(onPressed: _running ? null : _chooseOutput, icon: const Icon(Icons.save_alt), label: Text(_archivePath == null ? 'Choose Output' : 'Change Output')),
            const SizedBox(width: 8),
            FilledButton.icon(
              onPressed: _running || (kIsWeb ? _inputsWeb.isEmpty : _inputs.isEmpty) ? null : _runCreate,
              icon: const Icon(Icons.archive),
              label: Text(kIsWeb ? 'Create (download)' : 'Create'),
            ),
          ]),
          const SizedBox(height: 12),
          if (!kIsWeb && _archivePath != null) Text('Output: $_archivePath', style: const TextStyle(fontStyle: FontStyle.italic)),
          const SizedBox(height: 12),
          Expanded(
            child: DropArea(
              onDropped: (paths) => _addPaths(paths),
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Builder(builder: (_) {
                  final items = kIsWeb ? _inputsWeb.keys.toList() : _inputs;
                  if (items.isEmpty) {
                    return const Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(Icons.upload_file, size: 48, color: Colors.grey),
                          SizedBox(height: 8),
                          Text('Drag & drop files here'),
                          SizedBox(height: 4),
                          Text('Or use Add Files', style: TextStyle(color: Colors.grey)),
                        ],
                      ),
                    );
                  }
                  return ListView.separated(
                    itemCount: items.length,
                    separatorBuilder: (_, __) => const Divider(height: 1),
                    itemBuilder: (_, i) {
                      final name = items[i];
                      return ListTile(
                        dense: true,
                        title: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis),
                        leading: const Icon(Icons.insert_drive_file),
                        trailing: IconButton(
                          icon: const Icon(Icons.close),
                          tooltip: 'Remove',
                          onPressed: _running
                              ? null
                              : () => setState(() {
                                    if (kIsWeb) {
                                      _inputsWeb.remove(name);
                                    } else {
                                      _inputs.removeAt(i);
                                    }
                                  }),
                        ),
                      );
                    },
                  );
                }),
              ),
            ),
          ),
          const SizedBox(height: 12),
          LinearProgressIndicator(value: _running ? null : (_pct <= 0 ? null : _pct)),
          const SizedBox(height: 8),
          Row(children: [
            Expanded(child: Text(_status)),
            if (((!kIsWeb && _inputs.isNotEmpty) || (kIsWeb && _inputsWeb.isNotEmpty)) && !_running)
              TextButton.icon(
                onPressed: () => setState(() {
                  _inputs.clear();
                  _inputsWeb.clear();
                  _pct = 0;
                  _status = 'Idle';
                }),
                icon: const Icon(Icons.clear_all),
                label: const Text('Clear'),
              ),
          ]),
          if (_error != null) Text(_error!, style: const TextStyle(color: Colors.red)),
        ]),
      ),
    );
  }
}
