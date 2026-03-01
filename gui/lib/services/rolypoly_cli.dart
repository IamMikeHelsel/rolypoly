import 'dart:async';
import 'dart:convert';
import 'dart:io' show File, Platform, Process, ProcessResult;
import 'package:flutter/foundation.dart' show kIsWeb;

class RolyPolyCli {
  RolyPolyCli({String? binary}) : binary = _resolveBinary(binary);

  final String binary;

  static String _resolveBinary(String? provided) {
    if (provided != null && provided.isNotEmpty) return provided;
    final envOverride = Platform.environment['ROLYPOLY_CLI'];
    if (envOverride != null && envOverride.isNotEmpty) return envOverride;
    // On macOS, prefer a bundled CLI next to the app executable
    if (Platform.isMacOS) {
      try {
        final exe = File(Platform.resolvedExecutable);
        final candidate = File('${exe.parent.path}/rolypoly');
        if (candidate.existsSync()) return candidate.path;
      } catch (_) {}
    }
    return 'rolypoly';
  }

  Future<ProcessResult> create(String archive, List<String> files, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['create', archive, ...files, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> extract(String archive, String outDir, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['extract', archive, '-o', outDir, if (json) '--json'];
    return Process.run(binary, args);
  }

  Stream<Map<String, dynamic>> streamExtract(String archive, String outDir) async* {
    if (kIsWeb) throw UnsupportedError('Streaming process is unavailable on web');
    final args = ['extract', archive, '-o', outDir, '--json', '--progress'];
    final proc = await Process.start(binary, args);
    await for (final line in proc.stdout.transform(utf8.decoder).transform(const LineSplitter())) {
      try { yield jsonDecode(line) as Map<String, dynamic>; } catch (_) {}
    }
  }

  Future<ProcessResult> list(String archive, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['list', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<ProcessResult> validate(String archive, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['validate', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Stream<Map<String, dynamic>> streamValidate(String archive) async* {
    if (kIsWeb) throw UnsupportedError('Streaming process is unavailable on web');
    final args = ['validate', archive, '--json', '--progress'];
    final proc = await Process.start(binary, args);
    await for (final line in proc.stdout.transform(utf8.decoder).transform(const LineSplitter())) {
      try { yield jsonDecode(line) as Map<String, dynamic>; } catch (_) {}
    }
  }

  Future<ProcessResult> stats(String archive, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['stats', archive, if (json) '--json'];
    return Process.run(binary, args);
  }

  Future<Map<String, dynamic>?> listJson(String archive) async {
    final r = await list(archive, json: true);
    if (r.exitCode == 0) {
      try { return jsonDecode(r.stdout as String) as Map<String, dynamic>; } catch (_) {}
    }
    return null;
  }

  Future<Map<String, dynamic>?> statsJson(String archive) async {
    final r = await stats(archive, json: true);
    if (r.exitCode == 0) {
      try { return jsonDecode(r.stdout as String) as Map<String, dynamic>; } catch (_) {}
    }
    return null;
  }

  Future<ProcessResult> hash(String file, {bool json = false}) {
    if (kIsWeb) throw UnsupportedError('Process execution is unavailable on web');
    final args = ['hash', file, if (json) '--json'];
    return Process.run(binary, args);
  }

  /// Example of streaming progress (when `--json --progress` is implemented in CLI)
  Stream<Map<String, dynamic>> streamCreate(String archive, List<String> files) async* {
    if (kIsWeb) throw UnsupportedError('Streaming process is unavailable on web');
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
