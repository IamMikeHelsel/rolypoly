import 'dart:io';
import 'package:flutter/material.dart';
import '../services/rolypoly_cli.dart';

class ValidateStatsScreen extends StatefulWidget {
  const ValidateStatsScreen({super.key});
  @override
  State<ValidateStatsScreen> createState() => _ValidateStatsScreenState();
}

class _ValidateStatsScreenState extends State<ValidateStatsScreen> {
  final _cli = RolyPolyCli();
  String? _archive;
  String _validate = 'Unknown';
  Map<String, dynamic>? _stats;
  String _status = 'Idle';

  Future<void> _prepareSample() async {
    final tmp = Directory.systemTemp.createTempSync('rp');
    final f = File('${tmp.path}/large.txt')..writeAsStringSync('C' * 10000);
    final zip = '${tmp.path}/sample.zip';
    await _cli.create(zip, [f.path]);
    setState(() { _archive = zip; });
  }

  Future<void> _runValidate() async {
    if (_archive == null) return;
    setState(() { _status = 'Validating…'; _validate = 'Running'; });
    await for (final evt in _cli.streamValidate(_archive!)) {
      if (evt['event'] == 'done') {
        setState(() { _validate = 'OK'; _status = 'Validated'; });
      }
    }
  }

  Future<void> _runStats() async {
    if (_archive == null) return;
    setState(() { _status = 'Getting stats…'; });
    final data = await _cli.statsJson(_archive!);
    setState(() { _stats = data; _status = data != null ? 'Done' : 'Failed'; });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RolyPoly – Validate & Stats')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            ElevatedButton(onPressed: _prepareSample, child: const Text('Prepare Sample')),
            const SizedBox(width: 8),
            ElevatedButton(onPressed: _runValidate, child: const Text('Validate')),
            const SizedBox(width: 8),
            ElevatedButton(onPressed: _runStats, child: const Text('Stats')),
          ]),
          const SizedBox(height: 12),
          Text('Archive: ${_archive ?? '-'}'),
          const SizedBox(height: 12),
          Row(children: [
            Chip(label: Text('Validate: $_validate')),
          ]),
          const SizedBox(height: 12),
          if (_stats != null)
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Text('Files: ${_stats!['file_count'] ?? '-'}'),
                  Text('Directories: ${_stats!['dir_count'] ?? '-'}'),
                  Text('Uncompressed: ${_stats!['total_uncompressed_size'] ?? '-'} bytes'),
                  Text('Compressed: ${_stats!['total_compressed_size'] ?? '-'} bytes'),
                  Text('Ratio: ${_stats!['compression_ratio'] ?? '-'}%'),
                ]),
              ),
            ),
          const SizedBox(height: 8),
          Text(_status),
        ]),
      ),
    );
  }
}

