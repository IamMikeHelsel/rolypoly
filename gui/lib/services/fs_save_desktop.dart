import 'package:file_selector/file_selector.dart';

Future<String?> pickSaveZip({String suggestedName = 'archive.zip'}) async {
  return await getSavePath(
    suggestedName: suggestedName,
    acceptedTypeGroups: const [XTypeGroup(extensions: ['zip'])],
  );
}

