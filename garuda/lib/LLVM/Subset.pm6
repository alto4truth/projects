unit class LLVM::Subset:ver<0.0.1>:auth<local>;

has $.context-id;
has %.modules;
has %.functions;

submethod BUILD($? --> Nil) {
    $!context-id = 0;
}

class IRBuilder is export {
    method create-add($a, $b) { $a + $b }
    method create-sub($a, $b) { $a - $b }
    method create-mul($a, $b) { $a * $b }
    method create-icmp-eq($a, $b) { $a == $b }
    method create-icmp-ugt($a, $b) { $a > $b }
    method create-icmp-ult($a, $b) { $a < $b }
    method create-alloca($type) { "alloca_$type" }
    method create-load($ptr) { "load_$ptr" }
    method create-store($val, $ptr) { "store_$val" }
    method create-branch($label) { "br_$label" }
    method create-cond-branch($cond, $true-label, $false-label) { "br_cond_$cond" }
    method create-ret($val) { "ret_$val" }
    method create-call(&func, *@args) { "call_&func.name()" }
    method position-at-end($block) { }
}

class Function is export {
    has $.name;
    has @.blocks;
    has $.is-declaration = False;

    method get-name() { $.name }
    method size() { @.blocks.elems }
}

class BasicBlock is export {
    has $.name;
    has @.instructions;
    has $.parent-block;
}

class Module is export {
    has $.name;
    has %.functions;
    has %.globals;

    method get-name() { $.name }
    method size() { %.functions.elems }
    method global-size() { %.globals.elems }
    method get-function(Str $name) { %.functions{$name} }
    method get-or-create-function(Str $name) {
        %.functions{$name} //= Function.new(:name($name));
    }
}

class Type is export {
    method get-int-ty($context) { 'i32' }
    method get-float-ty($context) { 'float' }
    method get-double-ty($context) { 'double' }
    method get-pointer-ty($inner-type) { "ptr_$inner-type" }
    method get-void-ty($context) { 'void' }
}

class Value is export {
    has $.name;
    has $.type;
}

class ConstantInt is export {
    has Int $.value;
    has $.type;

    method new(Int $value, $type = 'i32') {
        self.bless(:$value, :$type);
    }
}

class ConstantFP is export {
    has Num $.value;
    has $.type;

    method new(Num $value, $type = 'double') {
        self.bless(:$value, :$type);
    }
}

sub parse-ir-file(Str $filename) is export {
    my $module = Module.new(:name($filename.IO.basename));
    my $dummy-func = Function.new(:name('dummy'));
    $module.functions{'dummy'} = $dummy-func;
    $module;
}

sub create-pass-manager() is export {
    my %pm;
    %pm<runs> = [];
    %pm;
}

sub add-analysis-pass(%pm, $pass) is export {
    %pm<runs>.push: $pass;
    %pm;
}

sub run-analyses(Module $module) is export {
    my %results;
    for $module.functions.values -> $func {
        %results{$func.name}<converged> = True;
        %results{$func.name}<iterations> = 1;
    }
    %results;
}

sub get-analysis-result(%pm, $analysis, $func) is export {
    my %res;
    %res<converged> = True;
    %res;
}

class PassBuilder is export {
    method add-simplify-cfg-pass() { }
    method add-inst-combine-pass() { }
    method add-gvn-pass() { }
    method build() { my %p; %p; }
}

class OptimizationRemark is export {
    has Str $.source-file;
    has Int $.line;
    has Str $.message;
}

class AnalysisResult is export {
    has %.group-domains;
    has $.converged = False;
    has $.iteration-count = 0;

    method set-group-domain(Str $name, $domain) {
        %.group-domains{$name} = $domain;
    }

    method set-converged(Bool $c) { $!converged = $c; }
    method set-iteration-count(Int $i) { $!iteration-count = $i; }
}

class FunctionAnalysis is export {
    method analyze-function(Function $func) {
        AnalysisResult.new();
    }
}

sub llvm-shutdown() is export { }

sub init-native-target() is export { True }
sub init-native-asmprinter() is export { True }
sub init-native-asmparser() is export { True }