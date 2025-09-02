import 'dart:async';
import 'dart:convert';
import 'dart:io';

class RolyPolyCli {
  RolyPolyCli({this.binary = 'rolypoly'});

  final String binary;

  Future<ProcessResult> create(String archive, List<String> files, {bool json = false}) {
    final args = ['create', archive, ...files, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> extract(String archive, String outDir, {bool json = false}) {
    final args = ['extract', archive, '-o', outDir, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> list(String archive, {bool json = false}) {
    final args = ['list', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> validate(String archive, {bool json = false}) {
    final args = ['validate', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> stats(String archive, {bool json = false}) {
    final args = ['stats', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> hash(String file, {bool json = false}) {
    final args = ['hash', file, if (json) '--json'];
    return Process.run(binary, args);
  }

  /// Example of streaming progress (when `--json --progress` is implemented in CLI)
  Stream<Map<String, dynamic>> streamCreate(String archive, List<String> files) async* {
    final args = ['create', archive, ...files, '--json', '--progress'];
    final proc = await Process.start(binary, args);
    await for (final line in proc.stdout.transform(utf8.decoder).transform(const LineSplitter())) {
      try {
        yield jsonDecode(line) as Map<String, dynamic>;
      } catch (_) {
        // ignore malformed lines
      }
    }
  }
}

