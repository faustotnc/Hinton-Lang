use crate::{lexer::tokens::Token, objects::Object};

#[derive(Clone)]
pub enum ASTNode {
    Module(ModuleNode),

    // Expressions
    Array(ArrayExprNode),
    ArrayIndexing(ArrayIndexingExprNode),
    Binary(BinaryExprNode),
    FunctionCall(FunctionCallExprNode),
    Identifier(IdentifierExprNode),
    Literal(LiteralExprNode),
    ObjectGetter(ObjectGetExprNode),
    ObjectSetter(ObjectSetExprNode),
    TernaryConditional(TernaryConditionalNode),
    Tuple(TupleExprNode),
    Unary(UnaryExprNode),
    VarReassignment(VarReassignmentExprNode),
    Instance(FunctionCallExprNode),

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
            _ => return false,
        }
    }

    pub fn is_false_literal(&self) -> bool {
        match self {
            ASTNode::Literal(x) => match x.value {
                Object::Bool(y) => !y,
                _ => false,
            },
            _ => return false,
        }
    }
}

#[derive(Clone)]
pub struct ModuleNode {
    pub body: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct LiteralExprNode {
    pub value: Object,
    pub token: Token,
}

#[derive(Clone)]
pub struct ArrayExprNode {
    pub values: Vec<Box<ASTNode>>,
    pub token: Token,
}

#[derive(Clone)]
pub struct TupleExprNode {
    pub values: Vec<Box<ASTNode>>,
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
    pub identifiers: Vec<Token>,
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
    None,   // a = b
}

#[derive(Clone)]
pub struct VarReassignmentExprNode {
    pub target: Token,
    pub value: Box<ASTNode>,
    pub opr_type: ReassignmentType,
    pub pos: (usize, usize),
}

#[derive(Clone)]
pub struct ArrayIndexingExprNode {
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
    pub body: Vec<ASTNode>,
    pub is_func_body: bool,
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
    pub body: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct LoopBranchStmtNode {
    pub token: Token,
    pub is_break: bool,
}

#[derive(Clone)]
pub struct FunctionDeclNode {
    pub name: Token,
    pub params: Vec<Parameter>,
    pub min_arity: u8,
    pub max_arity: u8,
    pub body: Box<ASTNode>,
    pub ends_with_return: bool,
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
    pub args: Vec<Argument>,
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
