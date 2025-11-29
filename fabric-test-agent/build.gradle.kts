plugins {
    id("fabric-loom") version "1.13-SNAPSHOT"
    id("org.jetbrains.kotlin.jvm") version "2.0.21"
    id("org.jetbrains.kotlin.plugin.serialization") version "2.0.21"
}

version = property("mod_version")!!
group = property("maven_group")!!

base {
    archivesName.set(property("archives_base_name") as String)
}

repositories {
    mavenCentral()
}

loom {
    // Client-only mod, don't split environments
}

dependencies {
    minecraft("com.mojang:minecraft:${property("minecraft_version")}")
    mappings("net.fabricmc:yarn:${property("yarn_mappings")}:v2")
    modImplementation("net.fabricmc:fabric-loader:${property("loader_version")}")
    modImplementation("net.fabricmc.fabric-api:fabric-api:${property("fabric_version")}")
    modImplementation("net.fabricmc:fabric-language-kotlin:1.12.3+kotlin.2.0.21")

    // JSON serialization
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")

    // Unix socket support (junixsocket)
    implementation("com.kohlschutter.junixsocket:junixsocket-core:2.10.1")
    include("com.kohlschutter.junixsocket:junixsocket-core:2.10.1")
    include("com.kohlschutter.junixsocket:junixsocket-common:2.10.1")
    include("com.kohlschutter.junixsocket:junixsocket-native-common:2.10.1")
}

tasks.processResources {
    inputs.property("version", project.version)

    filesMatching("fabric.mod.json") {
        expand("version" to project.version)
    }
}

tasks.withType<JavaCompile> {
    options.release.set(21)
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions.jvmTarget = "21"
}

java {
    withSourcesJar()
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

tasks.jar {
    from("LICENSE") {
        rename { "${it}_${project.base.archivesName.get()}" }
    }
}
