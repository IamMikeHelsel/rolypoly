import 'dart:io';
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';

class InspectScreen extends StatefulWidget {
  const InspectScreen({super.key});
  @override
  State<InspectScreen> createState() => _InspectScreenState();
}

class _InspectScreenState extends State<InspectScreen> {
  final _cli = RolyPolyCli();
  String? _archive;
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

  Future<void> _runList() async {
    if (_archive == null) return;
    setState(() { _status = 'Listing…'; _files = []; });
    final data = await _cli.listJson(_archive!);
    if (data != null) {
      setState(() { _files = List<String>.from(data['files'] ?? []); _status = 'Done'; });
    } else {
      setState(() { _status = 'Failed'; });
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
            ElevatedButton(onPressed: _prepareSample, child: const Text('Prepare Sample')),
            const SizedBox(width: 12),
            ElevatedButton(onPressed: _archive == null ? null : _runList, child: const Text('List')),
          ]),
          const SizedBox(height: 12),
          Text('Archive: ${_archive ?? '-'}'),
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

