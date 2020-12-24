package dev.alox.compiler.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.lexer.literalToken
import com.github.h0tk3y.betterParse.lexer.regexToken
import com.github.h0tk3y.betterParse.parser.*
import com.github.h0tk3y.betterParse.grammar.tryParseToEnd
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import dev.alox.compiler.Either
import dev.alox.compiler.ast.AstModule
import dev.alox.compiler.ast.AstModule.*
import dev.alox.compiler.ast.AstModule.Expression.*
import dev.alox.compiler.ast.Path
import dev.alox.compiler.report.Diagnostic
import dev.alox.compiler.report.Label
import dev.alox.compiler.report.SourceLocation

/**
 * The grammar for Alox defined using parser combinators
 */
object AstParser : Grammar<List<Declaration>>() {

    // LEXER TOKENS //

    // grouping tokens
    private val LEFT_PAREN by literalToken("(")
    private val RIGHT_PAREN by literalToken(")")
    private val LEFT_BRACKET by literalToken("[")
    private val RIGHT_BRACKET by literalToken("]")
    private val LEFT_BRACE by literalToken("{")
    private val RIGHT_BRACE by literalToken("}")

    // operators
    private val ADD by literalToken("+")
    private val SUB by literalToken("-")
    private val MUL by literalToken("*")
    private val DIV by literalToken("/")

    private val operator by ADD or SUB or MUL or DIV

    // misc symbols
    private val COMMA by literalToken(",")
    private val SEMICOLON by literalToken(";")
    private val EQUALS by literalToken("=")
    private val DOT by literalToken(".")
    private val COLON by literalToken(":")
    private val DOUBLE_COLON by literalToken("::")
    private val AMPERSAND by literalToken("&")

    // keywords
    private val IF by literalToken("if")
    private val ELSE by literalToken("else")
    private val ACTOR by literalToken("actor")
    private val STRUCT by literalToken("struct")
    private val FUNCTION by literalToken("fun")
    private val BEHAVIOR by literalToken("behave")
    private val KERNEL by literalToken("kernel")
    private val RETURN by literalToken("return")
    private val TRUE by literalToken("true")
    private val FALSE by literalToken("false")
    private val NEW by literalToken("new")
    private val LET by literalToken("let")
    private val THIS by literalToken("this")

    // literals
    private val INT_LITERAL by regexToken("\\d+")
    private val FLOAT_LITERAL by regexToken("\\d+\\.\\d+")
    private val CHAR_LITERAL by regexToken("'.'")
    private val STRING_LITERAL by regexToken("\".*?\"")

    private val WS by regexToken("\\s+", ignore = true)
    private val NEWLINE by regexToken("[\r\n]+", ignore = true)

    private val ID by regexToken("[A-Za-z0-9]+")

    private val name: Parser<String> by ID map { it.text }

    // PARSER RULES //

    // misc parts

    private val path: Parser<Path>
            by separatedTerms(name, DOUBLE_COLON, acceptZero = true) map { Path(it) }

    private val typeNamePart: Parser<Pair<Path, String>>
            by (path and -DOUBLE_COLON and name map { it.t1 to it.t2 }) or (name map { Path.empty to it })

    private val typeName: Parser<TypeName>
            by typeNamePart and optional(
                LEFT_BRACKET and separatedTerms(
                    parser(::typeName),
                    COMMA
                ) and RIGHT_BRACKET
            ) map {
                TypeName(it.t1.first, it.t1.second, it.t2?.t2.orEmpty())
            }

    private val typeParameters: Parser<List<TypeName>>
            by separatedTerms(typeName, COMMA)

    // expressions

    private val expressionList: Parser<List<Expression>>
            by separatedTerms(parser(::expression), COMMA, acceptZero = true)

    private val booleanLiteral: Parser<BooleanLiteral>
            by (TRUE or FALSE) map { BooleanLiteral(it.type == TRUE) }

    private val integerLiteral: Parser<IntegerLiteral>
            by INT_LITERAL map { IntegerLiteral(it.text.toLong()) }

    private val floatLiteral: Parser<FloatLiteral>
            by FLOAT_LITERAL map { FloatLiteral(it.text.toDouble()) }

    private val binaryOperator: Parser<BinaryOperator>
            by parser(::wrappedExpression) and operator and parser(::wrappedExpression) map { (lhs, operator, rhs) ->
                BinaryOperator(BinaryOperator.Kind.valueOf(operator.text), lhs, rhs)
            }

    private val variableReference: Parser<VariableReference>
            by typeNamePart map { (path, id) -> VariableReference(path, id) }

    private val thisExpression: Parser<This>
            by THIS map { This }

    private val addressOf: Parser<AddressOf>
            by -AMPERSAND and parser(::expression) map { AddressOf(it) }

    private val functionCall: Parser<FunctionCall>
            by parser(::wrappedExpression) and -LEFT_PAREN and expressionList and -RIGHT_PAREN map {
                FunctionCall(
                    it.t1,
                    it.t2
                )
            }

    private val methodCall: Parser<MethodCall>
            by parser(::wrappedExpression) and -DOT and name and -LEFT_PAREN and expressionList and -RIGHT_PAREN map {
                MethodCall(it.t1, it.t2, it.t3)
            }

    private val getField: Parser<GetField>
            by parser(::wrappedExpression) and -DOT and name map { GetField(it.t1, it.t2) }

    private val new: Parser<New>
            by -NEW and typeName map { New(it) }

    private val nonRecursiveExpression: Parser<Expression>
            by thisExpression or booleanLiteral or integerLiteral or floatLiteral or addressOf or variableReference

    private val expression: Parser<Expression>
            by methodCall or functionCall or getField or new or binaryOperator or nonRecursiveExpression

    private val wrappedExpression: Parser<Expression>
            by (-LEFT_PAREN and expression and -RIGHT_PAREN) or nonRecursiveExpression

    // statements

    private val variableDeclaration: Parser<Statement.VariableDeclaration>
            by -LET and name and -COLON and typeName map { (name, typeName) ->
                Statement.VariableDeclaration(name, typeName)
            }

    private val assignment: Parser<Statement.Assignment>
            by wrappedExpression and -EQUALS and expression map { (aggregate, value) ->
                Statement.Assignment(aggregate, value)
            }

    private val variableDefinition: Parser<Statement.VariableDefinition>
            by -LET and name and -COLON and typeName and -EQUALS and expression map { (name, typeName, value) ->
                Statement.VariableDefinition(name, typeName, value)
            }

    private val functionCallStatement: Parser<Statement.FunctionCall>
            by wrappedExpression and -LEFT_PAREN and expressionList and -RIGHT_PAREN map {
                Statement.FunctionCall(it.t1, it.t2)
            }

    private val methodCallStatement: Parser<Statement.MethodCall>
            by wrappedExpression and -DOT and name and -LEFT_PAREN and expressionList and -RIGHT_PAREN map {
                Statement.MethodCall(it.t1, it.t2, it.t3)
            }

    private val ifStatementPart
            by -IF and -LEFT_PAREN and expression and -RIGHT_PAREN and parser(::block) map {
                it.t1 to it.t2
            }

    private val elseIfStatement: Parser<Statement.IfStatement>
            by -ELSE and (parser(::ifStatement) or (parser(::block) map {
                Statement.IfStatement(BooleanLiteral(true), it, null)
            })) map { it }

    private val ifStatement: Parser<Statement.IfStatement>
            by ifStatementPart and optional(elseIfStatement) map {
                Statement.IfStatement(it.t1.first, it.t1.second, it.t2)
            }

    private val returnStatement: Parser<Statement.Return>
            by -RETURN and expression map { Statement.Return(it) }

    private val statement: Parser<Statement>
            by variableDefinition or variableDeclaration or assignment or returnStatement or methodCallStatement or functionCallStatement or ifStatement

    private val block: Parser<List<Statement>>
            by -LEFT_BRACE and zeroOrMore(statement) and -RIGHT_BRACE

    // declarations

    private val functionKind: Parser<Declaration.Function.Kind>
            by (BEHAVIOR or FUNCTION or KERNEL) map { Declaration.Function.Kind.from(it.text) }

    private val argument: Parser<Declaration.Function.Argument>
            by name and -COLON and typeName map { Declaration.Function.Argument(it.t1, it.t2) }

    private val argumentList: Parser<List<Declaration.Function.Argument>>
            by separatedTerms(argument, COMMA, acceptZero = true)

    private val function
            by loc { loc -> functionKind and name and
                    // generics
                    optional(-LEFT_BRACKET and separatedTerms(name, COMMA) and -LEFT_BRACKET) and
                    // args and body
                    -LEFT_PAREN and argumentList and -RIGHT_PAREN and optional(-COLON and typeName) and block map {
                Declaration.Function(
                    it.t2,
                    it.t1,
                    it.t3.orEmpty(),
                    it.t4,
                    it.t6,
                    it.t5 ?: TypeName(Path.empty, "Void", listOf()),
                    loc
                )
            } }

    private val structKind: Parser<Declaration.Struct.Kind>
            by (STRUCT or ACTOR) map { Declaration.Struct.Kind.valueOf(it.text.toUpperCase()) }

    private val field: Parser<Declaration.Struct.Field>
            by -LET and name and -COLON and typeName map { Declaration.Struct.Field(it.t1, it.t2) }

    private val fieldList: Parser<List<Declaration.Struct.Field>>
            by zeroOrMore(field)

    private val struct: Parser<Declaration.Struct>
            by loc {loc -> structKind and name and
                    // generics
                    optional(-LEFT_BRACKET and separatedTerms(name, COMMA) and -LEFT_BRACKET) and
                    -LEFT_BRACE and fieldList and zeroOrMore(function) and -RIGHT_BRACE map {
                Declaration.Struct(it.t2, it.t1, it.t3.orEmpty(), it.t4, it.t5, loc)
            } }

    private val declaration: Parser<Declaration>
            by function or struct

    override val rootParser: Parser<List<Declaration>>
            by oneOrMore(declaration)

    // END PARSING //

    fun parseModule(path: Path, name: String, source: String): Either<Diagnostic, AstModule> {
        return try {
            val declarations = AstParser.tryParseToEnd(source).toParsedOrThrow().value
            val module = AstModule(path, name, declarations, source)
            Either.Value(module)
        } catch (e: ParseException) {
            val labels = e.errorResult.toLabel(source)
            val diagnostic =
                Diagnostic(Diagnostic.Severity.ERROR, "Failed to parse module $name", labels.toMutableList())
            Either.Error(diagnostic)
        }
    }

}

// helper functions for better-parse

internal class LocationPreservingParser<T>(val parser: (SourceLocation) -> Parser<T>) : Parser<T> {
    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<T> =
        parser(tokens.getNotIgnored(fromPosition)?.toSourceLocation() ?: SourceLocation(0, 0, 0)).tryParse(
            tokens,
            fromPosition
        )
}

fun <T> loc(parser: (SourceLocation) -> Parser<T>): Parser<T> = LocationPreservingParser(parser)

fun TokenMatch.toSourceLocation(): SourceLocation = SourceLocation(row - 1, offset, length)

/**
 * Turns an ErrorResult to a set of our diagnostic labels
 */
fun ErrorResult.toLabel(source: String): List<Label> = when (this) {
    is MismatchedToken -> listOf(
        Label(
            source,
            found.toSourceLocation(),
            "Expected to find ${this.expected.name} but found ${this.found.type.name} instead"
        )
    )
    is UnparsedRemainder -> listOf(
        Label(
            source,
            startsWith.toSourceLocation(),
            "Couldn't parse ${startsWith.type.name}"
        )
    )
    is NoMatchingToken -> listOf(
        Label(
            source,
            tokenMismatch.toSourceLocation(),
            "Couldn't parse token from this symbol"
        )
    )
    is UnexpectedEof -> listOf(
        Label(
            source,
            SourceLocation(source.lines().size - 1, source.length - 2, 1),
            "Expected ${expected.name} but reach the end of the file instead"
        )
    )
    is AlternativesFailure -> {
        errors.flatMap { it.toLabel(source) }
    }
    else -> listOf(Label(source, SourceLocation(0, 0, 0), "Unknown error occurred while parsing"))
}
