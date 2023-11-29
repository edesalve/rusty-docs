"use client";
import React, { useState } from "react";
import { Header } from "@/components/Header";
import { ControlPanel } from "@/components/ControlPanel";
import { Logs } from "@/components/Logs";
import { Chat } from "@/components/Chat";
import { UserMsg } from "@/components/UserMsg";

// Main Assistant Component
export const Home = (props) => {
	const buttonFunctions = [
		() => parseRepository(payloads),
		() => documentRepository(payloads),
		() => embedRepository(payloads),
		() => askTheModel(payloads),
	];
	const buttonInputKeys = {
		0: ["repository_path", "write_to_json_path"],
		1: [
			"repository_path",
			"write_to_json_path",
			"write_inside_repository",
			"openai_api_key",
			"llm",
		],
		2: [
			"repository_path",
			"embedding_model",
			"qdrant_url",
			"qdrant_collection_name",
		],
		3: [
			"openai_api_key",
			"llm",
			"embedding_model",
			"qdrant_url",
			"qdrant_collection_name",
		],
	};
	const [chatMessages, setChatMessages] = useState([]);
	const [lastClickedButton, setLastClickedButton] = useState(0);
	const payloadsInit = {
		repository_path: "",
		write_to_json_path: "parsed_repository.json",
		write_inside_repository: "false",
		openai_api_key: "",
		llm: "gpt-4-1106-preview",
		embedding_model: "text-embedding-ada-002",
		qdrant_url: "",
		qdrant_collection_name: "my_qdrant_collection",
	};
	const [payloads, setPayloads] = useState(payloadsInit);
	const [showChat, setShowChat] = useState(false);
	const [showLogs, setShowLogs] = useState(false);

	const addChatMessage = (message) => {
		setChatMessages((prevMessages) => [...prevMessages, message]);
	};

	async function askTheModel(payloads, question) {
		const payload = {
		llm: payloads.llm,
		embedding_model: payloads.embedding_model,
		openai_api_key: payloads.openai_api_key,
		qdrant_collection_name: payloads.qdrant_collection_name,
		qdrant_url: payloads.qdrant_url,
		user_question: question,
		};

		return await call("ask", payload);
	}

	async function call(apiEndpoint, payload) {
		try {
			const response = await fetch(`http://localhost:8000/${apiEndpoint}`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(payload),
			});

			if (!response.ok) {
				const errorMessage = await response.text();
				throw new Error(
					`HTTP error! Status: ${response.status}, Message: ${errorMessage}`
				);
			}

			// Check if the response is a valid JSON
			const contentType = response.headers.get("content-type");
			if (contentType && contentType.includes("application/json")) {
				const data = await response.json();
				return data;
			} else {
				// If not valid JSON, return the response as text
				const data = await response.text();
				return data;
			}
		} catch (error) {
			console.error("Error calling endpoint:", error.message);
			throw error;
		}
	}

	async function documentRepository(payloads) {
		const payload = {
			llm: payloads.llm,
			openai_api_key: payloads.openai_api_key,
			repository_path: payloads.repository_path,
			write_inside_repository: payloads.write_inside_repository,
			write_to_json_path: payloads.write_to_json_path,
		};

		return await call("document", payload);
	}

	async function embedRepository(payloads) {
		const payload = {
			embedding_mode: payloads.embedding_model,
			openai_api_key: payloads.openai_api_key,
			repository_path: payloads.repository_path,
			qdrant_collection_name: payloads.qdrant_collection_name,
			qdrant_url: payloads.qdrant_url,
		};

		return await call("embed", payload);
	}

	const handleInputChange = (key, event) => {
		const updatedPayloads = { ...payloads, [key]: event.target.value };
		setPayloads(updatedPayloads);

		if (key === "write_inside_repository") {
			setWriteInsideRepository(event.target.value);
		} else if (key === "llm") {
			setLlm(event.target.value);
		} else if (key === "embedding_model") {
			setEmbeddingModel(event.target.value);
		}
	};

	async function parseRepository(payloads) {
		const payload = {
			repository_path: payloads.repository_path,
			write_to_json_path: payloads.write_to_json_path,
		};

		return await call("parse", payload);
	}
	
	const sendUserQuestion = async (question) => {
		try {
			const result = await askTheModel(payloads, question);
			addChatMessage(result.response);
		} catch (error) {
			addChatMessage(`Error: ${error.message}`);
		}
	};

	return (
		<main className="bg-docGrey3 flex-auto font-mono w-screen h-screen flex flex-col">
			<Header />
			<ControlPanel 
				buttonInputKeys={buttonInputKeys}
				handleInputChange={handleInputChange}
				initialPayloads={payloadsInit} 
				lastClickedButton={lastClickedButton}
				payloads={payloads} 
				setLastClickedButton={setLastClickedButton}
				setPayloads={setPayloads} 
				setShowChat={setShowChat}
				setShowLogs={setShowLogs}
			/>
			<div className={`mb-4 overflow-auto lg:px-32 px-12 scrollable-content text-docWhite ${showChat ? "hidden" : ""}`}>
				<Logs 
					buttonFunctions={buttonFunctions}
					buttonInputKeys={buttonInputKeys}
					lastClickedButton={lastClickedButton}
					payloads={payloads}
					setShowChat={setShowChat}
					setShowLogs={setShowLogs}
					showLogs={showLogs}
				/>
			</div>
			<div className={`mb-4 overflow-auto lg:px-32 px-12 scrollable-content text-docWhite ${showChat ? "" : "hidden"}`}>
				<Chat chatMessages={chatMessages} />
			</div>
			<div className={`mt-auto mb-10 lg:px-20 px-8 text-docWhite ${showChat ? "" : "hidden"}`}>
				<UserMsg 
					addChatMessage={addChatMessage}
					sendUserQuestion={sendUserQuestion} 
				/>
			</div>
			<style jsx>
				{`
					.scrollable-content::-webkit-scrollbar {
						display: none;
					}
				`}
			</style>
		</main>
	);
};

export default Home;
