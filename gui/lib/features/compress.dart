import 'dart:io';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';

class CompressScreen extends StatefulWidget {
  const CompressScreen({super.key});

  @override
  State<CompressScreen> createState() => _CompressScreenState();
}

class _CompressScreenState extends State<CompressScreen> {
  final _cli = RolyPolyCli();
  final List<String> _inputs = [];
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
    if (files.isNotEmpty) {
      _addPaths(files.map((e) => e.path));
    }
  }

  Future<void> _pickFolder() async {
    final dir = await getDirectoryPath();
    if (dir != null) _addPaths([dir]);
  }

  Future<void> _chooseOutput() async {
    final savePath = await getSavePath(suggestedName: 'archive.zip', acceptedTypeGroups: const [XTypeGroup(extensions: ['zip'])]);
    if (savePath != null) setState(() => _archivePath = savePath);
  }

  Future<void> _runCreate() async {
    if (_inputs.isEmpty) return;
    if (_running) return;
    setState(() {
      _running = true;
      _pct = 0;
      _status = 'Starting…';
      _error = null;
    });

    var out = _archivePath;
    if (out == null || out.isEmpty) {
      final picked = await getSavePath(suggestedName: 'archive.zip', acceptedTypeGroups: const [XTypeGroup(extensions: ['zip'])]);
      if (picked == null) {
        setState(() => _running = false);
        return;
      }
      out = picked;
      setState(() => _archivePath = out);
    }

    try {
      await for (final evt in _cli.streamCreate(out, _inputs)) {
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
            const SizedBox(width: 8),
            OutlinedButton.icon(onPressed: _running ? null : _pickFolder, icon: const Icon(Icons.create_new_folder), label: const Text('Add Folder')),
            const Spacer(),
            OutlinedButton.icon(onPressed: _running ? null : _chooseOutput, icon: const Icon(Icons.save_alt), label: Text(_archivePath == null ? 'Choose Output' : 'Change Output')),
            const SizedBox(width: 8),
            FilledButton.icon(onPressed: _running || _inputs.isEmpty ? null : _runCreate, icon: const Icon(Icons.archive), label: const Text('Create')),
          ]),
          const SizedBox(height: 12),
          if (_archivePath != null) Text('Output: $_archivePath', style: const TextStyle(fontStyle: FontStyle.italic)),
          const SizedBox(height: 12),
          Expanded(
            child: DropTarget(
              onDragEntered: (_) => setState(() => _dragging = true),
              onDragExited: (_) => setState(() => _dragging = false),
              onDragDone: (detail) {
                final paths = detail.files.map((f) => f.path).whereType<String>();
                _addPaths(paths);
                setState(() => _dragging = false);
              },
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: _dragging ? Colors.blue.withOpacity(0.06) : Colors.transparent,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: _dragging ? Colors.blue : Colors.grey.shade300, style: BorderStyle.solid),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: _inputs.isEmpty
                      ? const Center(
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Icon(Icons.upload_file, size: 48, color: Colors.grey),
                              SizedBox(height: 8),
                              Text('Drag & drop files or folders here'),
                              SizedBox(height: 4),
                              Text('Or use Add Files / Add Folder', style: TextStyle(color: Colors.grey)),
                            ],
                          ),
                        )
                      : ListView.separated(
                          itemCount: _inputs.length,
                          separatorBuilder: (_, __) => const Divider(height: 1),
                          itemBuilder: (_, i) {
                            final p = _inputs[i];
                            return ListTile(
                              dense: true,
                              title: Text(p, maxLines: 1, overflow: TextOverflow.ellipsis),
                              leading: const Icon(Icons.insert_drive_file),
                              trailing: IconButton(
                                icon: const Icon(Icons.close),
                                tooltip: 'Remove',
                                onPressed: _running
                                    ? null
                                    : () => setState(() {
                                          _inputs.removeAt(i);
                                        }),
                              ),
                            );
                          },
                        ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 12),
          LinearProgressIndicator(value: _running ? null : (_pct <= 0 ? null : _pct)),
          const SizedBox(height: 8),
          Row(children: [
            Expanded(child: Text(_status)),
            if (_inputs.isNotEmpty && !_running)
              TextButton.icon(
                onPressed: () => setState(() {
                  _inputs.clear();
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
