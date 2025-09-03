import 'dart:io' show Directory, File; // desktop only
import 'dart:typed_data';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';
import '../services/web_zip_read.dart';

class InspectScreen extends StatefulWidget {
  const InspectScreen({super.key});
  @override
  State<InspectScreen> createState() => _InspectScreenState();
}

class _InspectScreenState extends State<InspectScreen> {
  final _cli = RolyPolyCli();
  String? _archive;
  Uint8List? _webBytes;
  String? _webName;
  List<String> _files = [];
  String _status = 'Idle';

  Future<void> _prepareSample() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final a = File('${tmp.path}/a.txt')..writeAsStringSync('A');
    final b = File('${tmp.path}/b.txt')..writeAsStringSync('B');
    final zip = '${tmp.path}/sample.zip';
    await _cli.create(zip, [a.path, b.path]);
    setState(() { _archive = zip; });
  }

  Future<void> _pickArchive() async {
    final f = await openFile(acceptedTypeGroups: const [XTypeGroup(label: 'ZIP', extensions: ['zip'])]);
    if (f == null) return;
    if (kIsWeb) {
      _webBytes = await f.readAsBytes();
      _webName = f.name;
      setState(() {});
    } else {
      setState(() { _archive = f.path; });
    }
  }

  Future<void> _runList() async {
    if (!kIsWeb && _archive == null) return;
    setState(() { _status = 'Listing…'; _files = []; });
    if (kIsWeb) {
      if (_webBytes == null) { setState(() { _status = 'Pick a ZIP'; }); return; }
      try {
        final files = WebZipReadService().list(_webBytes!);
        setState(() { _files = files; _status = 'Done'; });
      } catch (e) {
        setState(() { _status = 'Failed: $e'; });
      }
    } else {
      final data = await _cli.listJson(_archive!);
      if (data != null) {
        setState(() { _files = List<String>.from(data['files'] ?? []); _status = 'Done'; });
      } else {
        setState(() { _status = 'Failed'; });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly – Inspect')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            if (!kIsWeb) ...[
              ElevatedButton(onPressed: _prepareSample, child: const Text('Prepare Sample')),
              const SizedBox(width: 12),
            ],
            OutlinedButton.icon(onPressed: _pickArchive, icon: const Icon(Icons.upload_file), label: const Text('Pick Archive')),
            const SizedBox(width: 12),
            ElevatedButton(onPressed: (!kIsWeb && _archive == null) && (kIsWeb && _webBytes == null) ? null : _runList, child: const Text('List')),
          ]),
          const SizedBox(height: 12),
          Text('Archive: ${kIsWeb ? (_webName ?? '-') : (_archive ?? '-') }'),
          const SizedBox(height: 12),
          Expanded(
            child: Container(
              decoration: BoxDecoration(border: Border.all(color: Colors.grey.shade300), borderRadius: BorderRadius.circular(8)),
              child: ListView.builder(
                itemCount: _files.length,
                itemBuilder: (ctx, i) => ListTile(title: Text(_files[i])),
              ),
            ),
          ),
          const SizedBox(height: 8),
          Text(_status),
        ]),
      ),
    );
  }
}
