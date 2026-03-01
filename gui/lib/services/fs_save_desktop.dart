import 'package:file_selector/file_selector.dart';

Future<String?> pickSaveZip({String suggestedName = 'archive.zip'}) async {
  final location = await getSaveLocation(
    suggestedName: suggestedName,
    acceptedTypeGroups: const [XTypeGroup(extensions: ['zip'])],
  );
  if (location == null) return null;
  // ignore: invalid_use_of_visible_for_testing_member
  return location.path;
}
