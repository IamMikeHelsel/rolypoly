import 'package:flutter/material.dart';

typedef PathsDropped = void Function(List<String> paths);

class DropArea extends StatelessWidget {
  final Widget child;
  final PathsDropped onDropped;
  final bool enabled;
  const DropArea({super.key, required this.child, required this.onDropped, this.enabled = true});

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.shade300),
        borderRadius: BorderRadius.circular(8),
      ),
      child: child,
    );
  }
}

