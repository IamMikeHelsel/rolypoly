import 'dart:typed_data';
import 'package:archive/archive.dart';
import 'web_download.dart';

class WebZipReadService {
  Archive _decode(Uint8List bytes) {
    final decoder = ZipDecoder();
    final archive = decoder.decodeBytes(bytes, verify: true);
    return archive;
  }

  List<String> list(Uint8List bytes) {
    final a = _decode(bytes);
    return a.files.map((f) => f.name).toList();
  }

  Map<String, dynamic> stats(Uint8List bytes) {
    final a = _decode(bytes);
    int fileCount = 0;
    int dirCount = 0;
    int uncompressed = 0;
    int compressed = 0;
    for (final f in a.files) {
      if (f.isFile) {
        fileCount++;
        uncompressed += f.size;
        // archive 3.x doesn't expose compressed size directly; approximate
        compressed += f.size;
      } else {
        dirCount++;
      }
    }
    final ratio = uncompressed > 0 ? (compressed / uncompressed) * 100.0 : 0.0;
    return {
      'file_count': fileCount,
      'dir_count': dirCount,
      'total_uncompressed_size': uncompressed,
      'total_compressed_size': compressed,
      'compression_ratio': ratio,
    };
  }

  bool validate(Uint8List bytes) {
    // decode with verify=true already checks CRC
    _decode(bytes);
    return true;
  }

  Future<void> extractAll(Uint8List bytes) async {
    final a = _decode(bytes);
    for (final f in a.files) {
      if (!f.isFile) continue;
      final data = f.content as List<int>;
      downloadBytes(Uint8List.fromList(data), f.name.split('/').last);
    }
  }

  Future<void> extractSelected(Uint8List bytes, List<String> names) async {
    if (names.isEmpty) return;
    final a = _decode(bytes);
    final wanted = names.toSet();
    for (final f in a.files) {
      if (!f.isFile) continue;
      if (!wanted.contains(f.name)) continue;
      final data = f.content as List<int>;
      downloadBytes(Uint8List.fromList(data), f.name.split('/').last);
    }
  }
}
