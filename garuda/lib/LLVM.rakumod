use LLVM::Subset;
use LLVM::Subset::Analysis;

our %EXPORT;

%EXPORT<LLVM::Subset> = %LLVM::Subset::EXPORT_DEFAULT;
%EXPORT<LLVM::Subset::Analysis> = %LLVM::Subset::Analysis::EXPORT_DEFAULT;