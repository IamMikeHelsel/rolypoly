import 'dart:typed_data';
import 'package:archive/archive.dart';

class WebZipService {
  Future<Uint8List> createZip(Map<String, Uint8List> files) async {
    final archive = Archive();
    files.forEach((name, bytes) {
      archive.addFile(ArchiveFile(name, bytes.length, bytes));
    });
    final data = ZipEncoder().encode(archive);
    return Uint8List.fromList(data ?? <int>[]);
  }
}

