unit class LLVM::Subset::Analysis:ver<0.0.1>:auth<local>;

use LLVM::Subset;

class GroupDomain is export {
    has Str $.name;
    has Bool $.is-top = False;
    has Bool $.is-bottom = False;

    method set-top() { $!is-top = True; }
    method set-bottom() { $!is-bottom = True; }
    method is-top() { $!is-top }
    method is-bottom() { $!is-bottom }
    method join(GroupDomain $other) {
        my $result = GroupDomain.new(:name($.name));
        $result.set-top if $.is-top || $other.is-top;
        $result;
    }
    method meet(GroupDomain $other) {
        my $result = GroupDomain.new(:name($.name));
        $result.set-bottom if $.is-bottom || $other.is-bottom;
        $result;
    }
}

class GroupDomainManager is export {
    has %.domains;
    has %.worklist;

    method get-or-create-group(Str $name) {
        %.domains{$name} //= GroupDomain.new(:name($name));
    }

    method add-to-worklist(GroupDomain $domain) {
        %.worklist{$domain.name} = $domain;
    }

    method process-worklist() {
        my $changed = False;
        for %.worklist.values -> $domain {
            $changed = True;
        }
        $changed;
    }

    method get-results() { %.domains; }
}

class ProgressUpdateTracker is export {
    has Int $.update-count = 0;
    has %.updates;
    has %.convergence-info;

    method record-update(Str $location, $value) {
        $!update-count++;
        %.updates{$location} = $value;
    }

    method has-converged(Str $key) {
        %.convergence-info{$key}<converged> //= False;
    }

    method mark-converged(Str $key) {
        %.convergence-info{$key}<converged> = True;
    }

    method get-update-count() { $!update-count; }
}

class ProgressUpdate is export {
    has Str $.location;
    has $.old-value;
    has $.new-value;
    has Int $.timestamp;

    method changed() { $.old-value !=== $.new-value; }
}

class SubsequentProgressUpdate is export {
    has %.chained-updates;

    method add-chained-update($update) {
        %.chained-updates.push: $update;
    }

    method get-chained-updates() { %.chained-updates; }
}

class ProgressUpdateNotifier is export {
    has %.listeners;
    has %.history;

    method subscribe($listener) {
        my $id = %.listeners.elems;
        %.listeners{$id} = $listener;
        $id;
    }

    method notify(ProgressUpdate $update) {
        %.history.push: $update;
        for %.listeners.values -> $listener {
            $listener.receive-update($update) if $listener.^can('receive-update');
        }
    }

    method get-history() { %.history; }
}

class AbstractDomain is export {
    has Bool $.is-top = False;
    has Bool $.is-bottom = False;

    method is-top() { $!is-top }
    method is-bottom() { $!is-bottom }
    method top() { self but role { has $.is-top = True } }
    method bottom() { self but role { has $.is-bottom = True } }
}

class ValueDomain is export {
    has $.value;
    has Bool $.is-unknown = False;

    method is-unknown() { $!is-unknown }
    method get-value() { $!value; }
}

class IntervalDomain is export {
    has $.lower;
    has $.upper;
    has Bool $.is-unbounded = False;

    method new($lower = -Inf, $upper = Inf) {
        self.bless(:$lower, :$upper);
    }

    method get-lower() { $!lower }
    method get-upper() { $!upper }

    method widen-with(IntervalDomain $other) {
        my $new-lower = $!is-unbounded ?? -Inf !! ($!lower < $other.lower ?? $!lower !! $other.lower);
        my $new-upper = $!is-unbounded ?? Inf !! ($!upper > $other.upper ?? $!upper !! $other.upper);
        IntervalDomain.new($new-lower, $new-upper);
    }

    method join-with(IntervalDomain $other) {
        my $new-lower = min($!lower, $other.lower);
        my $new-upper = max($!upper, $other.upper);
        IntervalDomain.new($new-lower, $new-upper);
    }
}

class IntegerDomain is export {
    has Int $.value;
    has Bool $.is-top = False;
    has Bool $.is-bottom = False;

    method new(Int $value = 0) {
        self.bless(:$value);
    }

    method top() {
        my $d = IntegerDomain.new(0);
        $d.is-top = True;
        $d;
    }
}

class ConstantDomain is export {
    has $.constant;

    method new($c) {
        self.bless(:constant($c));
    }

    method get-constant() { $.constant; }
}

class BooleanDomain is export {
    has Bool $.value;
    has Bool $.is-undefined = False;

    method new(Bool $value = False) {
        self.bless(:$value);
    }

    method is-true() { $.value && !$.is-undefined; }
    method is-false() { !$.value && !$.is-undefined; }
    method is-undefined() { $!is-undefined; }
}

class PointerDomain is export {
    has Str $.target;
    has Bool $.is-null = False;

    method new(Str $target = '') {
        self.bless(:$target);
    }

    method set-null() { $!is-null = True; }
    method is-null() { $!is-null; }
}

class FloatDomain is export {
    has Num $.value;
    has Bool $.is-nan = False;
    has Bool $.is-infinite = False;

    method new(Num $value = 0e0) {
        self.bless(:$value);
    }
}

class GroupDomainVisitor is export {
    has %.visited;
    has %.results;

    method visit(GroupDomain $domain) {
        %.visited{$domain.name} = True;
        %.results{$domain.name} = $domain;
    }

    method has-visited(Str $name) { %.visited{$name} //= False; }
    method get-results() { %.results; }
}

class GroupDomainBuilder is export {
    has %.partial-domains;

    method add-element(Str $group, $element) {
        %.partial-domains{$group} //= [];
        %.partial-domains{$group}.push: $element;
    }

    method build(Str $group-name) {
        my $domain = GroupDomain.new(:name($group-name));
        $domain.set-top;
        $domain;
    }
}