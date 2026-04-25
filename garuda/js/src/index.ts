export enum DomainType {
  Top = "Top",
  Bottom = "Bottom",
  Integer = "Integer",
  Pointer = "Pointer",
  Boolean = "Boolean",
  Float = "Float",
  Interval = "Interval",
  Constant = "Constant",
  Struct = "Struct",
  Array = "Array",
  Group = "Group",
}

export enum DomainOperator {
  Join = "Join",
  Meet = "Meet",
  Widen = "Widen",
  Narrow = "Narrow",
  Assign = "Assign",
  CompareEq = "CompareEq",
  CompareNe = "CompareNe",
  CompareLt = "CompareLt",
  CompareLe = "CompareLe",
  CompareGt = "CompareGt",
  CompareGe = "CompareGe",
  Add = "Add",
  Sub = "Sub",
  Mul = "Mul",
  Div = "Div",
  Mod = "Mod",
  And = "And",
  Or = "Or",
  Xor = "Xor",
  Not = "Not",
  Shl = "Shl",
  Shr = "Shr",
}

export interface DomainState {
  isTop: boolean;
  isBottom: boolean;
  domainType: DomainType;
}

export type Domain =
  | TopDomain
  | BooleanDomain
  | IntegerDomain
  | PointerDomain
  | IntervalDomain
  | FloatDomain
  | ConstantDomain
  | GroupDomain;

export class TopDomain {
  public isTop: boolean = true;
  public isBottom: boolean = false;

  public toString(): string {
    return "⊤";
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Top,
    };
  }
}

export class BooleanDomain {
  public value: boolean | null = null;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(value?: boolean) {
    if (value !== undefined) {
      this.value = value;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    return this.value !== null ? String(this.value) : "bool";
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Boolean,
    };
  }

  public static newBottom(): BooleanDomain {
    return new class extends BooleanDomain {
      constructor() {
        super();
        this.isTop = false;
        this.isBottom = true;
      }
    }();
  }
}

export class IntegerDomain {
  public lowerBound: number = -Infinity;
  public upperBound: number = Infinity;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(lower?: number, upper?: number) {
    if (lower !== undefined && upper !== undefined) {
      this.lowerBound = lower;
      this.upperBound = upper;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    if (this.lowerBound === this.upperBound) return String(this.lowerBound);
    return `[${this.lowerBound}, ${this.upperBound}]`;
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Integer,
    };
  }

  public static newBottom(): IntegerDomain {
    return new class extends IntegerDomain {
      constructor() {
        super();
        this.isTop = false;
        this.isBottom = true;
      }
    }();
  }
}

export class PointerDomain {
  public address: number | null = null;
  public isNull: boolean = false;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(address?: number) {
    if (address !== undefined) {
      this.address = address;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public static null(): PointerDomain {
    const p = new PointerDomain();
    p.isNull = true;
    p.isTop = false;
    return p;
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    if (this.isNull) return "null";
    if (this.address !== null) return `ptr(${this.address})`;
    return "ptr";
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Pointer,
    };
  }
}

export class IntervalDomain {
  public lower: number = -Infinity;
  public upper: number = Infinity;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(lower?: number, upper?: number) {
    if (lower !== undefined && upper !== undefined) {
      this.lower = lower;
      this.upper = upper;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    if (this.lower === this.upper) return String(this.lower);
    return `[${this.lower}, ${this.upper}]`;
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Interval,
    };
  }
}

export class FloatDomain {
  public lowerBound: number = -Infinity;
  public upperBound: number = Infinity;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(lower?: number, upper?: number) {
    if (lower !== undefined && upper !== undefined) {
      this.lowerBound = lower;
      this.upperBound = upper;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    if (this.lowerBound === this.upperBound) return String(this.lowerBound);
    return `[${this.lowerBound}, ${this.upperBound}]`;
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Float,
    };
  }
}

export type ConstantValue = number | boolean | string;

export class ConstantDomain {
  public value: ConstantValue | null = null;
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(value?: ConstantValue) {
    if (value !== undefined) {
      this.value = value;
      this.isTop = false;
      this.isBottom = false;
    }
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    return this.value !== null ? String(this.value) : "constant";
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Constant,
    };
  }
}

export class GroupDomain {
  public name: string = "";
  public members: Map<string, Domain> = new Map();
  public isTop: boolean = true;
  public isBottom: boolean = false;

  constructor(name?: string) {
    if (name) {
      this.name = name;
      this.isTop = false;
    }
  }

  public addMember(key: string, domain: Domain): void {
    this.members.set(key, domain);
    this.isTop = false;
  }

  public getMember(key: string): Domain | undefined {
    return this.members.get(key);
  }

  public toString(): string {
    if (this.isTop) return "⊤";
    if (this.isBottom) return "⊥";
    return `Group(${this.name})`;
  }

  public getState(): DomainState {
    return {
      isTop: this.isTop,
      isBottom: this.isBottom,
      domainType: DomainType.Group,
    };
  }
}

export class DomainFactory {
  public static create(domainType: DomainType): Domain {
    switch (domainType) {
      case DomainType.Top:
        return new TopDomain();
      case DomainType.Boolean:
        return new BooleanDomain();
      case DomainType.Integer:
        return new IntegerDomain();
      case DomainType.Pointer:
        return new PointerDomain();
      case DomainType.Interval:
        return new IntervalDomain();
      case DomainType.Float:
        return new FloatDomain();
      case DomainType.Constant:
        return new ConstantDomain();
      case DomainType.Group:
        return new GroupDomain();
      default:
        return new TopDomain();
    }
  }
}

export interface AnalysisResult {
  violations: Array<[string, string]>;
  warnings: Array<[string, string]>;
  converged: boolean;
  iterationCount: number;
}

export class GarudaAnalysis {
  public name: string = "";
  public converged: boolean = false;
  public iterationCount: number = 0;

  constructor(name?: string) {
    if (name) this.name = name;
  }

  public analyze(): AnalysisResult {
    return {
      violations: [],
      warnings: [],
      converged: this.converged,
      iterationCount: this.iterationCount++,
    };
  }
}

export class ProgressUpdateTracker {
  public updates: Array<{stage: string; message: string; percentage: number}> = [];

  public addUpdate(stage: string, message: string, percentage: number): void {
    this.updates.push({ stage, message, percentage });
  }
}