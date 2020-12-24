package dev.alox.compiler.report

/**
 * A location of something in the source code
 */
data class SourceLocation(val line: Int, val offset: Int, val length: Int)

/**
 * A label for a source location in a diagnostic
 */
data class Label(val source: String, val sourceLocation: SourceLocation, val message: String) {
    /**
     * Convert this label to a formatted string to be printed
     */
    override fun toString(): String {
        // TODO: develop this further
        val line = sourceLocation.line
        val lines = source.lines()
        val charsBefore = (0 until line).sumBy { lines[it].length + 1 }
        val inlineOffset = sourceLocation.offset - charsBefore

        val maxLineNumSize = ((line - 1)..(line + 1)).map { it.toString().length }.max()!!

        val beginningOfFile = line == 0
        val endOfFile = line == lines.size - 1

        val contextLines = lines.subList(if (beginningOfFile) line else line - 1, if (endOfFile) line + 1 else line + 2)
        val mainIndex = if (beginningOfFile) 0 else 1

        // build lines
        val firstLine = if (beginningOfFile) "" else "${(line).toString().padStart(maxLineNumSize, ' ')}| ${contextLines[0]}"
        val mainLine = "${(line + 1).toString().padStart(maxLineNumSize, ' ')}| ${contextLines[mainIndex]}"
        val arrowLine = " ".repeat(maxLineNumSize) + "| " + "-".repeat(inlineOffset) + "^".repeat(sourceLocation.length)
        val lastLine = if (endOfFile) "" else "${(line + 2).toString().padStart(maxLineNumSize, ' ')}| ${contextLines[2]}"
        val messageLine = " ".repeat(maxLineNumSize) + "= $message"

        return "$firstLine\n$mainLine\n$arrowLine\n$lastLine\n$messageLine"
    }
}

/**
 * A message for a diagnostic that isn't tied to a source location
 */
data class Note(val message: String)

data class Diagnostic(
    val severity: Severity,
    val message: String,
    val labels: MutableList<Label> = mutableListOf(),
    val notes: MutableList<Note> = mutableListOf()
) {

    enum class Severity {
        NOTE, WARNING, ERROR
    }

    override fun toString(): String {
        return labels.joinToString("\n") { it.toString() }
    }

}
