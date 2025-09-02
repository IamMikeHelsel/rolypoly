import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';

typedef PathsDropped = void Function(List<String> paths);

class DropArea extends StatefulWidget {
  final Widget child;
  final PathsDropped onDropped;
  final bool enabled;
  const DropArea({super.key, required this.child, required this.onDropped, this.enabled = true});

  @override
  State<DropArea> createState() => _DropAreaState();
}

class _DropAreaState extends State<DropArea> {
  bool _drag = false;
  @override
  Widget build(BuildContext context) {
    return DropTarget(
      onDragEntered: (_) => setState(() => _drag = true),
      onDragExited: (_) => setState(() => _drag = false),
      onDragDone: (d) {
        final paths = d.files.map((f) => f.path).whereType<String>().toList();
        if (widget.enabled && paths.isNotEmpty) {
          widget.onDropped(paths);
        }
        setState(() => _drag = false);
      },
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: _drag ? Colors.blue.withOpacity(0.06) : Colors.transparent,
          border: Border.all(color: _drag ? Colors.blue : Colors.grey.shade300),
          borderRadius: BorderRadius.circular(8),
        ),
        child: widget.child,
      ),
    );
  }
}

