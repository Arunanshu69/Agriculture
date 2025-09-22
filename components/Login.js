import React, { useState } from "react";
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Image,
  Alert,
} from "react-native";
import RNPickerSelect from "react-native-picker-select";

export const Login=() =>{
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [role, setRole] = useState("");

  const handleLogin = () => {
    if (!email || !password || !role) {
      Alert.alert("Missing Fields", "Please fill in all fields before logging in.");
      return;
    }
    // 🚀 Proceed with actual login logic (API call, navigation, etc.)
    Alert.alert("Success", `Logged in as ${role}`);
  };

  return (
    <View style={styles.container}>
      {/* Logo */}
      <View style={{ marginBottom: 30, alignItems: "center" }}>
        <Image
          source={{ uri: "https://img.icons8.com/ios-filled/100/4a90e2/shield.png" }}
          style={{ width: 60, height: 60 }}
        />
      </View>

      <Text style={styles.title}>Hello.</Text>

      {/* Email */}
      <TextInput
        style={styles.input}
        placeholder="Email"
        value={email}
        onChangeText={setEmail}
      />

      {/* Password */}
      <TextInput
        style={styles.input}
        placeholder="Password"
        secureTextEntry
        value={password}
        onChangeText={setPassword}
      />

      {/* Dropdown (Role) */}
      <RNPickerSelect
        onValueChange={(value) => setRole(value)}
        items={[
          { label: "Farmer", value: "farmer" },
          { label: "Admin", value: "admin" },
          { label: "Customer", value: "customer" },
        ]}
        style={{
          inputAndroid: styles.input,
          inputIOS: styles.input,
          placeholder: { color: "#999" },
        }}
        placeholder={{ label: "Select Role...", value: null }}
        value={role}
      />

      {/* Forgot password */}
      <TouchableOpacity style={{ alignSelf: "flex-end", marginBottom: 20 }}>
        <Text style={styles.link}>Forgot your password?</Text>
      </TouchableOpacity>

      {/* Login Button */}
      <TouchableOpacity style={styles.button} onPress={handleLogin}>
        <Text style={styles.buttonText}>Log In</Text>
      </TouchableOpacity>

      {/* Sign Up */}
      <Text style={styles.footerText}>
        You do not have an account yet?{" "}
        <Text style={styles.link}>Create!</Text>
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#faf7ff",
    justifyContent: "center",
    alignItems: "center",
    padding: 20,
  },
  title: {
    fontSize: 22,
    fontWeight: "600",
    marginBottom: 20,
    color: "#333",
  },
  input: {
    width: "100%",
    borderWidth: 1,
    borderColor: "#ccc",
    borderRadius: 8,
    padding: 12,
    marginBottom: 15,
    backgroundColor: "#fff",
  },
  button: {
    width: "100%",
    backgroundColor: "#4a40e2",
    padding: 15,
    borderRadius: 25,
    alignItems: "center",
    marginBottom: 20,
  },
  buttonText: {
    color: "#fff",
    fontSize: 16,
    fontWeight: "600",
  },
  link: {
    color: "#4a40e2",
    fontWeight: "500",
  },
  footerText: {
    fontSize: 14,
    color: "#555",
  },
});

