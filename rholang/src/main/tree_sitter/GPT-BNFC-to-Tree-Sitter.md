**user**:
Can you translate the following BNFC grammar to the equivalent tree-sitter grammar: -- Custom integer token according to https://github.com/BNFC/bnfc/issues/153#issuecomment-152612231
-- notice this shadows the built-in Integer token, which might be a problem if we need integers
-- in the grammar for other purposes than the integer literal.

position token LongLiteral digit+;

-- A program is just a process. We put the coercions first, because that's what
-- bnfc uses to determine the starting production.
_. Proc  ::= Proc1 ;
_. Proc1 ::= Proc2 ;
_. Proc2 ::= Proc3 ;
_. Proc3 ::= Proc4 ;
_. Proc4 ::= Proc5 ;
_. Proc5 ::= Proc6 ;
_. Proc6 ::= Proc7 ;
_. Proc7 ::= Proc8 ;
_. Proc8 ::= Proc9 ;
_. Proc9 ::= Proc10 ;
_. Proc10 ::= Proc11 ;
_. Proc11 ::= Proc12 ;
_. Proc12 ::= Proc13 ;
_. Proc13 ::= Proc14 ;
_. Proc14 ::= Proc15 ;
_. Proc15 ::= Proc16 ;
_. Proc16 ::= "{" Proc "}" ;

-- Processes
-- In general the expression style processes are higher precedence.
-- Expression style is anything that necessary resolves to a single ground value
-- or a collection.
PGround.         Proc16 ::= Ground ;
PCollect.        Proc16 ::= Collection ;
PVar.            Proc16 ::= ProcVar ;
PVarRef.         Proc13 ::= VarRefKind Var ;
PNil.            Proc16 ::= "Nil" ;
PSimpleType.     Proc16 ::= SimpleType ;
PNegation.       Proc15 ::= "~" Proc15 ;
PConjunction.    Proc14 ::= Proc14 "/\\" Proc15 ;
PDisjunction.    Proc13 ::= Proc13 "\\/" Proc14 ;
PEval.           Proc12 ::= "*" Name ;
PMethod.         Proc11 ::= Proc11 "." Var "(" [Proc] ")" ;
PExprs.          Proc11 ::= "(" Proc4 ")" ;
PNot.            Proc10 ::= "not" Proc10 ;
PNeg.            Proc10 ::= "-" Proc10 ;
PMult.           Proc9  ::= Proc9 "*" Proc10 ;
PDiv.            Proc9  ::= Proc9 "/" Proc10 ;
PMod.            Proc9  ::= Proc9 "%" Proc10 ;
PPercentPercent. Proc9  ::= Proc9 "%%" Proc10 ;
PAdd.            Proc8  ::= Proc8 "+" Proc9 ;
PMinus.          Proc8  ::= Proc8 "-" Proc9 ;
PPlusPlus.       Proc8  ::= Proc8 "++" Proc9 ;
PMinusMinus.     Proc8  ::= Proc8 "--" Proc9 ;
PLt.             Proc7  ::= Proc7 "<" Proc8 ;
PLte.            Proc7  ::= Proc7 "<=" Proc8 ;
PGt.             Proc7  ::= Proc7 ">" Proc8 ;
PGte.            Proc7  ::= Proc7 ">=" Proc8 ;
PMatches.        Proc6  ::= Proc7 "matches" Proc7 ;
PEq.             Proc6  ::= Proc6 "==" Proc7 ;
PNeq.            Proc6  ::= Proc6 "!=" Proc7 ;
PAnd.            Proc5  ::= Proc5 "and" Proc6 ;
POr.             Proc4  ::= Proc4 "or" Proc5 ;
PSend.           Proc3  ::= Name Send "(" [Proc] ")" ;
PContr.          Proc2  ::= "contract" Name "(" [Name] NameRemainder")" "=" "{" Proc "}" ;
PInput.          Proc2  ::= "for" "(" [Receipt] ")" "{" Proc "}" ;
PChoice.         Proc2  ::= "select" "{" [Branch] "}" ;
PMatch.          Proc2  ::= "match" Proc4 "{" [Case] "}" ;
PBundle.         Proc2  ::= Bundle "{" Proc "}" ;
PLet.            Proc2  ::= "let" Decl Decls "in" "{" Proc "}" ;
PIf.             Proc1  ::= "if" "(" Proc ")" Proc2 ;
-- Use precedence to force braces around an interior if.
PIfElse.      Proc1 ::= "if" "(" Proc ")" Proc2 "else" Proc1 ;
PNew.         Proc1 ::= "new" [NameDecl] "in" Proc1 ;
PSendSynch.   Proc1 ::= Name "!?" "(" [Proc] ")" SynchSendCont ;
PPar.         Proc  ::= Proc "|" Proc1 ;

separator Proc "," ;

-- Let declarations
DeclImpl.    Decl ::= [Name] NameRemainder "<-" [Proc] ;

LinearDeclImpl.    LinearDecl ::= Decl ;
separator nonempty LinearDecl ";" ;

ConcDeclImpl.    ConcDecl ::= Decl ;
separator nonempty ConcDecl "&" ;

EmptyDeclImpl.      Decls ::= ;
LinearDeclsImpl.    Decls ::= ";" [LinearDecl] ;
ConcDeclsImpl.      Decls ::= "&" [ConcDecl] ;

-- Continuation for synchronous send

EmptyCont.      SynchSendCont ::= "." ;
NonEmptyCont.   SynchSendCont ::= ";" Proc1 ;

-- Process variables
ProcVarWildcard. ProcVar ::= "_" ;
ProcVarVar.      ProcVar ::= Var ;

-- Names
NameWildcard. Name ::= "_" ;
NameVar.      Name ::= Var ;
NameQuote.    Name ::= "@" Proc12 ;
separator Name "," ;

-- Bundle
BundleWrite.    Bundle ::= "bundle+" ;
BundleRead.     Bundle ::= "bundle-" ;
BundleEquiv.    Bundle ::= "bundle0" ;
BundleReadWrite. Bundle ::= "bundle"  ;

-- Receipt
ReceiptLinear.    Receipt ::= ReceiptLinearImpl ;
ReceiptRepeated.  Receipt ::= ReceiptRepeatedImpl ;
ReceiptPeek.      Receipt ::= ReceiptPeekImpl ;
separator nonempty Receipt ";" ;

-- Linear Receipts
LinearSimple. ReceiptLinearImpl ::= [LinearBind] ;
-- Implementing this will be tricky.
-- for (x <- a; y <- b if *x)
-- LinearCond.   Linear ::= [LinearBind] "if" Proc ;

-- Single Linear Bind
LinearBindImpl. LinearBind ::= [Name] NameRemainder "<-" NameSource ;
separator nonempty LinearBind "&" ;

SimpleSource.         NameSource ::= Name ;
ReceiveSendSource.    NameSource ::= Name "?!" ;
SendReceiveSource.    NameSource ::= Name "!?" "(" [Proc] ")" ;

-- Repeated Receipts
RepeatedSimple. ReceiptRepeatedImpl ::= [RepeatedBind] ;
-- Single Repeated Bind
RepeatedBindImpl. RepeatedBind ::= [Name] NameRemainder "<=" Name ;
separator nonempty RepeatedBind "&" ;

-- Peek Receipts
PeekSimple. ReceiptPeekImpl ::= [PeekBind] ;
-- Single Peek
PeekBindImpl. PeekBind ::= [Name] NameRemainder "<<-" Name ;
separator nonempty PeekBind "&" ;

-- Types of Send:
SendSingle.   Send ::= "!" ;
SendMultiple. Send ::= "!!" ;

-- Select Branches
BranchImpl. Branch ::= ReceiptLinearImpl "=>" Proc3 ;
separator nonempty Branch "" ;

-- Match Cases
CaseImpl. Case ::= Proc13 "=>" Proc3 ;
separator nonempty Case "" ;

-- Name Declarations.
-- Eventually will have IOPairs.
NameDeclSimpl. NameDecl ::= Var ;
NameDeclUrn. NameDecl ::= Var "(" UriLiteral ")" ;
separator nonempty NameDecl "," ;

-- Booleans:
BoolTrue.   BoolLiteral ::= "true" ;
BoolFalse.  BoolLiteral ::= "false" ;
-- Ground types:
-- The "Literal" suffix avoids collisions with Simple Types
GroundBool.    Ground ::= BoolLiteral ;
GroundInt.     Ground ::= LongLiteral ;
GroundString.  Ground ::= StringLiteral ;
GroundUri.     Ground ::= UriLiteral ;
token StringLiteral ( '"' ((char - ["\"\\"]) | ('\\' ["\"\\nt"]))* '"' );
token UriLiteral ('`' ((char - ["\\`"]) | ('\\' ["`\\"]))* '`') ;
-- Collections:
CollectList.   Collection ::= "[" [Proc] ProcRemainder "]" ;
CollectTuple.  Collection ::= Tuple;
CollectSet.    Collection ::= "Set" "(" [Proc] ProcRemainder")" ;
CollectMap.    Collection ::= "{" [KeyValuePair] ProcRemainder"}" ;
KeyValuePairImpl.  KeyValuePair ::= Proc ":" Proc ;
separator KeyValuePair "," ;

TupleSingle.    Tuple ::= "(" Proc ",)" ;
TupleMultiple.  Tuple ::= "(" Proc "," [Proc] ")" ;

-- Remainders:
ProcRemainderVar.   ProcRemainder ::= "..." ProcVar ;
ProcRemainderEmpty. ProcRemainder ::= "" ;
NameRemainderVar.   NameRemainder ::= "..." "@" ProcVar ;
NameRemainderEmpty. NameRemainder ::= "" ;

-- VarRefKind:
VarRefKindProc. VarRefKind ::= "=" ;
VarRefKindName. VarRefKind ::= "=" "*" ;

-- Simple Types:
SimpleTypeBool. SimpleType ::= "Bool" ;
SimpleTypeInt. SimpleType ::= "Int" ;
SimpleTypeString. SimpleType ::= "String" ;
SimpleTypeUri. SimpleType ::= "Uri" ;
SimpleTypeByteArray. SimpleType ::= "ByteArray" ;

token Var (((letter | '\'') (letter | digit | '_' | '\'')*)|(('_') (letter | digit | '_' | '\'')+)) ;

-- Comments:
comment "//" ;
comment "/*" "*/" ;


--------------------

**assistant**:
Here's the equivalent Tree-sitter grammar for the BNFC grammar you provided:

```javascript
;; Custom integer token according to https://github.com/BNFC/bnfc/issues/153#issuecomment-152612231
;; notice this shadows the built-in Integer token, which might be a problem if we need integers
;; in the grammar for other purposes than the integer literal.

(LongLiteral) @number

;; A program is just a process. We put the coercions first, because that's what
;; bnfc uses to determine the starting production.
(Proc) = (Proc1)
(Proc1) = (Proc2)
(Proc2) = (Proc3)
(Proc3) = (Proc4)
(Proc4) = (Proc5)
(Proc5) = (Proc6)
(Proc6) = (Proc7)
(Proc7) = (Proc8)
(Proc8) = (Proc9)
(Proc9) = (Proc10)
(Proc10) = (Proc11)
(Proc11) = (Proc12)
(Proc12) = (Proc13)
(Proc13) = (Proc14)
(Proc14) = (Proc15)
(Proc15) = (Proc16)
(Proc16) = "{" @open_curly (Proc) @close_curly

;; Processes
(PGround) = (Ground)
(PCollect) = (Collection)
(PVar) = (ProcVar)
(PVarRef) = (VarRefKind) (Var)
(PNil) = "Nil"
(PSimpleType) = (SimpleType)
(PNegation) = "~" @unary (Proc15)
(PConjunction) = (Proc14) "/\\" @binary (Proc15)
(PDisjunction) = (Proc13) "\\/" @binary (Proc14)
(PEval) = "*" @unary (Name)
(PMethod) = (Proc11) "." (Var) "(" @call (Proc*) @close_paren
(PExprs) = "(" @call (Proc4) @close_paren
(PNot) = "not" @unary (Proc10)
(PNeg) = "-" @unary (Proc10)
(PMult) = (Proc9) "*" @binary (Proc10)
(PDiv) = (Proc9) "/" @binary (Proc10)
(PMod) = (Proc9) "%" @binary (Proc10)
(PPercentPercent) = (Proc9) "%%" @binary (Proc10)
(PAdd) = (Proc8) "+" @binary (Proc9)
(PMinus) = (Proc8) "-" @binary (Proc9)
(PPlusPlus) = (Proc8) "++" @binary (Proc9)
(PMinusMinus) = (Proc8) "--" @binary (Proc9)
(PLt) = (Proc7) "<" @binary (Proc8)
(PLte) = (Proc7) "<=" @binary (Proc8)
(PGt) = (Proc7) ">" @binary (Proc8)
(PGte) = (Proc7) ">=" @binary (Proc8)
(PMatches) = (Proc7) "matches" @binary (Proc7)
(PEq) = (Proc6) "==" @binary (Proc7)
(PNeq) = (Proc6) "!=" @binary (Proc7)
(PAnd) = (Proc5) "and" @binary (Proc6)
(POr) = (Proc4) "or" @binary (Proc5)
(PSend) = (Name) (Send) "(" @call (Proc*) @close_paren
(PContr) = "contract" (Name) "(" @call (Name*) (NameRemainder) @close_paren "=" "{" @open_curly (Proc) @close_curly
(PInput) = "for" "(" @call (Receipt*) @close_paren "{" @open_curly (Proc) @close_curly
(PChoice) = "select" "{" @open_curly (Branch*) @close_curly
(PMatch) = "match" @call (Proc4) "{" @open_curly (Case*) @close_curly
(PBundle) = (Bundle) "{" @open_curly (Proc) @close_curly
(PLet) = "let" (Decl) (Decls) "in" "{" @open_curly (Proc) @close_curly
(PIf) = "if" "(" @call (Proc) @close_paren (Proc2)
(PIfElse) = "if" "(" @call (Proc) @close_paren (Proc2) "else" (Proc1)
(PNew) = "new" @call (NameDecl*) "in" (Proc1)
(PSendSynch) = (Name) "!?" "(" @call (Proc*) @close_paren (SynchSendCont)
(PPar) = (Proc) "|" (Proc1)

(Decl) = (NameRemainder) "<-" @binary (Proc*)
(LinearDecl) = (Decl)
(ConcDecl) = (Decl)
(Decls) = ";" @separator (LinearDecl*)
(Decls) = "&" @separator (ConcDecl*)

(SynchSendCont) = "." @separator
(SynchSendCont) = ";" (Proc1)

(ProcVarWildcard) = "_"
(ProcVarVar) = (Var)

(NameWildcard) = "_"
(NameVar) = (Var)
(NameQuote) = "@" (Proc12)

(BundleWrite) = "bundle+"
(BundleRead) = "bundle-"
(BundleEquiv) = "bundle0"
(BundleReadWrite) = "bundle"

(ReceiptLinear) = (ReceiptLinearImpl)
(ReceiptRepeated) = (ReceiptRepeatedImpl)
(ReceiptPeek) = (ReceiptPeekImpl)

(LinearSimple) = (LinearBind*)
(LinearBindImpl) = (NameRemainder) "<-" (NameSource)
(SimpleSource) = (Name)
(ReceiveSendSource) = (Name) "?!"
(SendReceiveSource) = (Name) "!?" "(" @call (Proc*) @close_paren

(RepeatedSimple) = (RepeatedBind*)
(RepeatedBindImpl) = (NameRemainder) "<=" (Name)

(PeekSimple) = (PeekBind*)
(PeekBindImpl) = (NameRemainder) "<<-" (Name)

(SendSingle) = "!"
(SendMultiple) = "!!"

(BranchImpl) = (ReceiptLinearImpl) "=>" (Proc3)

(CaseImpl) = (Proc13) "=>" (Proc3)

(NameDeclSimpl) = (Var)
(NameDeclUrn) = (Var) "(" @call (UriLiteral) @close_paren

(BoolTrue) = "true"
(BoolFalse) = "false"

(GroundBool) = (BoolLiteral)
(GroundInt) = (LongLiteral)
(GroundString) = (StringLiteral)
(GroundUri) = (UriLiteral)

(StringLiteral) = "\"" @string (( /[^"\\]/ | /\\./ )* ) "\""
(UriLiteral) = "`" @string (( /[^`\\]/ | /\\./ )* ) "`"

(CollectList) = "[" @list (Proc*) (ProcRemainder) "]"
(CollectTuple) = (Tuple)
(CollectSet) = "Set" "(" @set (Proc*) (ProcRemainder) ")"
(CollectMap) = "{" @map (KeyValuePair*) (ProcRemainder) "}"
(KeyValuePairImpl) = (Proc) ":" @pair (Proc)

(TupleSingle) = "(" (Proc) ",)"
(TupleMultiple) = "(" (Proc) "," @tuple (Proc*) ")"

(ProcRemainderVar) = "..." (ProcVar)
(ProcRemainderEmpty) = ""
(NameRemainderVar) = "..." "@" (ProcVar)
(NameRemainderEmpty) = ""

(VarRefKindProc) = "="
(VarRefKindName) = "=" "*"

(SimpleTypeBool) = "Bool"
(SimpleTypeInt) = "Int"
(SimpleTypeString) = "String"
(SimpleTypeUri) = "Uri"
(SimpleTypeByteArray) = "ByteArray"

(Var) = /((([a-zA-Z]|')|'_')([a-zA-Z]|[0-9]|'_'|'\')*)|(((_)([a-zA-Z]|[0-9]|'_'|'\')+))/

;; Comments
(comment) = "//" /.*/ "\n"
(comment) = "/*" @comment /(.|[\r\n])*?/ "*/"
```

This Tree-sitter grammar closely follows the structure of the BNFC grammar you provided. The main differences are:

1. The use of Tree-sitter's field and query syntax to represent the different production rules and their relationships.
2. The use of regular expressions to define the `Var` token, which is a bit more complex than the BNFC version.
3. The handling of comments using Tree-sitter's built-in comment syntax.

Overall, the Tree-sitter grammar should be able to parse the same language as the BNFC grammar, with the added benefit of providing a more structured and queryable representation of the abstract syntax tree.