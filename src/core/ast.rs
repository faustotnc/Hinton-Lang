use crate::core::tokens::Token;
use crate::objects::Object;

#[derive(Clone)]
pub enum ASTNode {
   Module(ModuleNode),

   // Expressions
   Array(ArrayExprNode),
   Binary(BinaryExprNode),
   Dictionary(DictionaryExprNode),
   FunctionCall(FunctionCallExprNode),
   Identifier(IdentifierExprNode),
   Instance(FunctionCallExprNode),
   Literal(LiteralExprNode),
   ObjectGetter(ObjectGetExprNode),
   ObjectSetter(ObjectSetExprNode),
   SelfExpr(SelfExprNode),
   Subscript(SubscriptExprNode),
   SubscriptAssignment(SubscriptAssignExprNode),
   TernaryConditional(TernaryConditionalNode),
   Tuple(TupleExprNode),
   Unary(UnaryExprNode),
   VarReassignment(VarReassignmentExprNode),
   Lambda(FunctionDeclNode),

   // Declarations
   ClassDecl(ClassDeclNode),
   ConstantDecl(ConstantDeclNode),
   FunctionDecl(FunctionDeclNode),
   VariableDecl(VariableDeclNode),

   // Statements
   BlockStmt(BlockNode),
   ExpressionStmt(ExpressionStmtNode),
   ForStmt(ForStmtNode),
   IfStmt(IfStmtNode),
   LoopBranch(LoopBranchStmtNode),
   ReturnStmt(ReturnStmtNode),
   WhileStmt(WhileStmtNode),
}

impl ASTNode {
   pub fn is_truthy_literal(&self) -> bool {
      match self {
         ASTNode::Literal(x) => !x.value.is_falsey(),
         _ => false,
      }
   }

   pub fn is_false_literal(&self) -> bool {
      match self {
         ASTNode::Literal(x) => match x.value {
            Object::Bool(y) => !y,
            _ => false,
         },
         _ => false,
      }
   }
}

#[derive(Clone)]
pub struct ModuleNode {
   pub body: Box<[ASTNode]>,
}

#[derive(Clone)]
pub struct LiteralExprNode {
   pub value: Object,
   pub token: Token,
}

#[derive(Clone)]
pub struct ArrayExprNode {
   pub values: Box<[ASTNode]>,
   pub token: Token,
}

#[derive(Clone)]
pub struct TupleExprNode {
   pub values: Box<[ASTNode]>,
   pub token: Token,
}

#[derive(Clone)]
pub struct DictionaryExprNode {
   pub keys: Box<[Token]>,
   pub values: Box<[ASTNode]>,
   pub token: Token,
}

#[derive(Clone)]
pub struct UnaryExprNode {
   pub operand: Box<ASTNode>,
   pub opr_type: UnaryExprType,
   pub pos: (usize, usize),
}

#[derive(Clone, Debug)]
pub enum UnaryExprType {
   ArithmeticNeg,
   LogicNeg,
   BitwiseNeg,
}

#[derive(Clone)]
pub struct BinaryExprNode {
   pub left: Box<ASTNode>,
   pub right: Box<ASTNode>,
   pub opr_token: Token,
   pub opr_type: BinaryExprType,
}

#[derive(Clone, Debug)]
pub enum BinaryExprType {
   Addition,
   BitwiseAND,
   BitwiseOR,
   BitwiseShiftLeft,
   BitwiseShiftRight,
   BitwiseXOR,
   Division,
   Expo,
   LogicAND,
   LogicEQ,
   LogicGreaterThan,
   LogicGreaterThanEQ,
   LogicLessThan,
   LogicLessThanEQ,
   LogicNotEQ,
   LogicOR,
   Minus,
   Modulus,
   Multiplication,
   Nullish,
   Range,
}

#[derive(Clone)]
pub struct TernaryConditionalNode {
   pub condition: Box<ASTNode>,
   pub true_branch_token: Token,
   pub branch_true: Box<ASTNode>,
   pub false_branch_token: Token,
   pub branch_false: Box<ASTNode>,
}

#[derive(Clone)]
pub struct IdentifierExprNode {
   pub token: Token,
}

#[derive(Clone)]
pub struct ExpressionStmtNode {
   pub child: Box<ASTNode>,
   pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct VariableDeclNode {
   pub identifiers: Box<[Token]>,
   pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub enum ReassignmentType {
   Plus,   // a += b
   Minus,  // a -= b
   Mul,    // a *= b
   Div,    // a /= b
   Expo,   // a **= b
   Mod,    // a %= b
   ShiftL, // a <<= b
   ShiftR, // a =>> b
   BitAnd, // a &= b
   Xor,    // a ^= b
   BitOr,  // a |= b
   Assign, // a = b
}

#[derive(Clone)]
pub struct VarReassignmentExprNode {
   pub target: Token,
   pub value: Box<ASTNode>,
   pub opr_type: ReassignmentType,
   pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct SubscriptExprNode {
   pub target: Box<ASTNode>,
   pub index: Box<ASTNode>,
   pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ConstantDeclNode {
   pub name: Token,
   pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub struct BlockNode {
   pub body: Box<[ASTNode]>,
   pub end_of_block: Token,
}

#[derive(Clone)]
pub struct IfStmtNode {
   pub condition: Box<ASTNode>,
   pub then_token: Token,
   pub then_branch: Box<ASTNode>,
   pub else_branch: Box<Option<ASTNode>>,
   pub else_token: Option<Token>,
}

#[derive(Clone)]
pub struct WhileStmtNode {
   pub token: Token,
   pub condition: Box<ASTNode>,
   pub body: Box<ASTNode>,
}

#[derive(Clone)]
pub struct ForStmtNode {
   pub token: Token,
   pub id: IdentifierExprNode,
   pub iterator: Box<ASTNode>,
   pub body: Box<[ASTNode]>,
}

#[derive(Clone)]
pub struct LoopBranchStmtNode {
   pub token: Token,
   pub is_break: bool,
}

#[derive(Clone)]
pub struct FunctionDeclNode {
   pub name: Token,
   pub params: Box<[Parameter]>,
   pub arity: (u8, u8),
   pub body: Box<[ASTNode]>,
}

#[derive(Clone)]
pub struct Parameter {
   pub name: Token,
   pub is_optional: bool,
   pub default: Option<Box<ASTNode>>,
}

#[derive(Clone)]
pub struct FunctionCallExprNode {
   pub target: Box<ASTNode>,
   pub args: Box<[Argument]>,
   pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct Argument {
   pub name: Option<ASTNode>,
   pub is_named: bool,
   pub value: Box<ASTNode>,
}

#[derive(Clone)]
pub struct ReturnStmtNode {
   pub token: Token,
   pub value: Option<Box<ASTNode>>,
}

#[derive(Clone)]
pub struct ClassDeclNode {
   pub name: Token,
   pub members: Box<[ClassMemberDeclNode]>,
}

#[derive(Clone)]
pub enum ClassMemberDecl {
   Var(VariableDeclNode),
   Const(ConstantDeclNode),
   Method(FunctionDeclNode),
}

#[derive(Clone)]
pub struct ClassMemberDeclNode {
   pub member_type: ClassMemberDecl,
}

#[derive(Clone)]
pub struct SelfExprNode {
   pub token: Token,
}

#[derive(Clone)]
pub struct ObjectGetExprNode {
   pub target: Box<ASTNode>,
   pub getter: Token,
}

#[derive(Clone)]
pub struct ObjectSetExprNode {
   pub target: Box<ASTNode>,
   pub setter: Token,
   pub value: Box<ASTNode>,
   pub opr_type: ReassignmentType,
}

#[derive(Clone)]
pub struct SubscriptAssignExprNode {
   pub target: Box<ASTNode>,
   pub index: Box<ASTNode>,
   pub value: Box<ASTNode>,
   pub pos: (usize, usize),
   pub opr_type: ReassignmentType,
}
