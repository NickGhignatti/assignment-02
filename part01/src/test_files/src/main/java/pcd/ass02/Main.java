package pcd.ass02;

import java.io.File;
import com.github.javaparser.StaticJavaParser;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.ImportDeclaration;
import com.github.javaparser.ast.PackageDeclaration;
import com.github.javaparser.ast.body.ClassOrInterfaceDeclaration;
import com.github.javaparser.ast.body.FieldDeclaration;
import com.github.javaparser.ast.body.MethodDeclaration;
import com.github.javaparser.ast.body.VariableDeclarator;
import com.github.javaparser.ast.expr.ObjectCreationExpr;
import com.github.javaparser.ast.type.TypeParameter;
import com.github.javaparser.ast.visitor.VoidVisitorAdapter;

public class Main {

	public static void main(String[] args) throws Exception  {

		File file = new File("src/main/java/pcd/ass02/MyClass.java");
		
		CompilationUnit cu = StaticJavaParser.parse(file);
				
		new VoidVisitorAdapter<Object>() {
            
			/**
			 *  Finding a type in a class/interface declaration 
			 */			
			public void visit(ClassOrInterfaceDeclaration n, Object arg) {
                super.visit(n, arg);
                System.out.println("type " + n.getName() + " (class/int decl)");
            }
            
			/**
			 *  Package declaration 
			 */			
            public void visit(PackageDeclaration n, Object arg) {
                super.visit(n, arg);
                System.out.println("package " + n.getName() + " (package decl)");
            }
            
			/**
			 *  Finding a type in a field declaration 
			 */			
            public void visit(FieldDeclaration n, Object arg) {
                super.visit(n, arg);
                VariableDeclarator vd = (VariableDeclarator) n.getChildNodes().get(0);
                System.out.println("type " + vd.getType().asString() + " (field decl)");
            }
            
			/**
			 *  Finding types in methods declaration 
			 */			
            public void visit(MethodDeclaration n, Object arg) {
                super.visit(n, arg);
                // System.out.println("method: " + n.toString());
                for (var p: n.getParameters()) {
                    System.out.println("type " + p.getType().asString() + " (method decl, param type)");
                }
                System.out.println("return type: " + n.getType().asString() + " (method decl, return type)");
            }
            
			/**
			 *  Finding type in object creation 
			 */			
            public void visit(ObjectCreationExpr n, Object arg) {
                super.visit(n, arg);
                var interfaceOrClassType =  n.getChildNodes().get(0);
                System.out.println("type " + interfaceOrClassType + " (obj creation decl)");

            }
            
			/**
			 *  Finding types in variable declaration 
			 */			
            public void visit(VariableDeclarator n, Object arg) {
                super.visit(n, arg);
                var t = n.getType();
                System.out.println("type " + n.getType().asString() + " (var decl)");
            }

			/**
			 *  Finding types in type parameter 
			 */			
            public void visit(TypeParameter n, Object arg) {
                super.visit(n, arg);
                System.out.println("type " + n.asString() +"(type decl)");
            }

			/**
			 *  Finding types in import declaration 
			 */			
            public void visit(ImportDeclaration n, Object arg) {
                super.visit(n, arg);
                if (!n.isAsterisk()) {
	                var typeName =  n.getChildNodes().get(0);
	                var packageName = typeName.getChildNodes().get(0);
	                System.out.println( "type " + typeName + " package: " + packageName + " (import)");
                } else {
                	var packageName = n.getChildNodes().get(0);
                	System.out.println( "package " + packageName + " (import)");
                }
            }
        }.visit(cu,null);
        

	}

}
